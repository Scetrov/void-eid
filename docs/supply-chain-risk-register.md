# Supply Chain Risk Register

This register tracks dependency risks that remain after the AppSec remediation pass and should be reviewed during dependency maintenance.

## Open Rust advisories

Generated with `cd src/backend && cargo audit --json`.

| Advisory | Package | Status | Owner/action |
| --- | --- | --- | --- |
| RUSTSEC-2026-0002 | `lru` | Partially blocked by Sui graph. `lru 0.16.4` is patched, but `0.10.1` and `0.13.0` remain through Sui/anemo and Sui Move runtime crates. | Backend/Sui owner: move to a Sui revision that resolves `lru >=0.16.3` everywhere or remove the affected Sui dependency path. |
| RUSTSEC-2026-0097 | `rand` | Compatible instances were updated to `0.8.6` and `0.9.4`; `0.10.1` is already patched. Any remaining audit report should be checked against the dependency path before release. | Backend owner: re-run `cargo tree -i rand --locked` after every Sui update and keep all non-Sui instances on patched versions. |
| RUSTSEC-2023-0071 | `rsa` | Transitive advisory, likely from the Sui dependency graph. | Backend/Sui owner: identify path with `cargo tree -i rsa --locked`; update or remove the upstream path if reachable from runtime code. |
| RUSTSEC-2025-0141 | `bincode` | Unmaintained transitive dependency. | Backend/Sui owner: identify path and update upstream when compatible. |
| RUSTSEC-2024-0388 | `derivative` | Unmaintained transitive dependency. | Backend/Sui owner: identify path and update upstream when compatible. |
| RUSTSEC-2025-0057 | `fxhash` | Unmaintained transitive dependency. | Backend/Sui owner: identify path and update upstream when compatible. |
| RUSTSEC-2024-0384 | `instant` | Unmaintained transitive dependency. | Backend/Sui owner: identify path and update upstream when compatible. |
| RUSTSEC-2024-0436 | `paste` | Unmaintained transitive dependency. | Backend/Sui owner: identify path and update upstream when compatible. |
| RUSTSEC-2024-0370 | `proc-macro-error` | Unmaintained transitive dependency. | Backend/Sui owner: identify path and update upstream when compatible. |
| RUSTSEC-2024-0320 | `yaml-rust` | Unmaintained transitive dependency. | Backend/Sui owner: identify path and update upstream when compatible. |

## Pinning policy

- GitHub Actions external `uses:` references must stay pinned to full commit SHAs.
- Cargo git dependencies must include an explicit `rev` in `Cargo.toml`; do not rely only on `Cargo.lock` for git source pinning.
- Runtime and deployment images should use digest-pinned references.
- Package manager installs in CI and Dockerfiles must use lock-enforcing modes (`bun install --frozen-lockfile`, `cargo build --locked`, `cargo install --locked --version ...`).

## Current exception

`mysten-metrics 0.7.0` remains through the Sui git dependency graph. It must be removed by moving to a compatible Sui revision that no longer resolves it or by replacing the affected Sui dependency path.
