//! Subprocess opt-in checks for the stub API binary.
//!
//! The stub API must only start when `STUB_API=1`. These tests launch the real
//! binary in a controlled environment (temp working directory so `dotenvy`
//! cannot find the repo's `.env`) and verify that any value other than exactly
//! `"1"` exits non-zero *before* the listener binds, while `"1"` reaches
//! readiness.
//!
//! `CARGO_BIN_EXE_stub_api` resolves to the built `stub_api` binary for this
//! package, so these tests run under `cargo test` with no extra tooling.

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn stub_exe() -> String {
    env!("CARGO_BIN_EXE_stub_api").to_string()
}

/// Create a unique temporary working directory.
fn make_tmp_dir() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let d = std::env::temp_dir().join(format!("stub_optin_{}_{}", std::process::id(), n));
    std::fs::create_dir_all(&d).expect("create temp working dir");
    d
}

/// Spawn the stub binary with only `STUB_API` (per-case) and `PORT` controlled;
/// the rest of the environment is inherited so the dynamic linker still resolves
/// its libraries. The temp `cwd` keeps `dotenvy` from loading a repo `.env`.
/// `stub_api: Some(v)` sets `STUB_API=v`; `None` removes any inherited value.
fn spawn_stub(stub_api: Option<&str>, cwd: &PathBuf, port: u16) -> std::process::Child {
    let mut cmd = Command::new(stub_exe());
    cmd.current_dir(cwd)
        .env("PORT", port.to_string())
        .env("IDENTITY_HASH_PEPPER", "test-pepper")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    match stub_api {
        Some(v) => {
            let _ = cmd.env("STUB_API", v);
        }
        None => {
            cmd.env_remove("STUB_API");
        }
    }
    cmd.spawn().expect("spawn stub_api binary")
}

/// Poll `try_wait` until the child exits or `timeout` elapses.
fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Option<std::process::ExitStatus> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait().ok()? {
            Some(status) => return Some(status),
            None => {
                if Instant::now() >= deadline {
                    return None;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

fn drain_owned<R: Read>(mut handle: R) -> String {
    let mut s = String::new();
    let _ = handle.read_to_string(&mut s);
    s
}

/// Every non-`"1"` value must refuse to start: non-zero exit, before any
/// listener binds, with a clear message.
#[test]
fn stub_refuses_without_exact_opt_in() {
    let cases: [(Option<&str>, &str); 4] = [
        (None, "unset"),
        (Some(""), "empty"),
        (Some("0"), "zero"),
        (Some("2"), "invalid"),
    ];

    for (value, label) in cases {
        let cwd = make_tmp_dir();
        let mut child = spawn_stub(value, &cwd, 0);

        let status = wait_with_timeout(&mut child, Duration::from_secs(15))
            .unwrap_or_else(|| panic!("stub ({label}) did not exit within timeout"));

        let stdout = child.stdout.take().map(drain_owned).unwrap_or_default();
        let stderr = child.stderr.take().map(drain_owned).unwrap_or_default();

        assert!(
            !status.success(),
            "stub ({label}) must exit non-zero (got {:?})",
            status.code()
        );
        assert!(
            !stdout.contains("Stub API Listening"),
            "stub ({label}) must not reach the listener; stdout: {}",
            stdout
        );
        assert!(
            stderr.contains("refusing to start"),
            "stub ({label}) must explain the refusal; stderr: {}",
            stderr
        );

        let _ = std::fs::remove_dir_all(&cwd);
    }
}

/// With exactly `STUB_API=1` the stub must reach readiness (bind + announce).
#[test]
fn stub_starts_when_exactly_opted_in() {
    let cwd = make_tmp_dir();
    // PORT=0 binds an ephemeral port, avoiding any collision with other tests.
    let mut child = spawn_stub(Some("1"), &cwd, 0);

    let stdout = child.stdout.take().expect("stdout piped");
    let (tx, rx) = std::sync::mpsc::channel::<bool>();
    std::thread::spawn(move || {
        use std::io::BufRead;
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(l) if l.contains("Stub API Listening") => {
                    let _ = tx.send(true);
                    break;
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });

    let deadline = Instant::now() + Duration::from_secs(20);
    let ready = loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(true) => break true,
            _ => {
                // Bail out early if the process died unexpectedly.
                if child.try_wait().ok().flatten().is_some() {
                    break false;
                }
                if Instant::now() >= deadline {
                    break false;
                }
            }
        }
    };

    assert!(ready, "stub with STUB_API=1 should reach readiness");

    child.kill().expect("kill stub process");
    let _ = child.wait();
    let _ = std::fs::remove_dir_all(&cwd);
}
