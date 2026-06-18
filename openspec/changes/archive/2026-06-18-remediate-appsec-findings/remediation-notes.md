# Security Remediation Notes

## Pre-change Dependabot alert list

The Dependabot alerts endpoint returned `404 Not Found` for the active GitHub token, so the pre-change list was recorded from the provided dependency vulnerability report:

- Critical: `mysten-metrics` Rust package in `src/backend/Cargo.lock`, removed from crates.io for malicious code.
- High: five `aws-lc-sys` / AWS-LC findings in `src/backend/Cargo.lock`.
- Low: `@babel/core` arbitrary file read in `src/frontend/package-lock.json`.
- Low: `lru` / `IterMut` Stacked Borrows finding in `src/backend/Cargo.lock`.
- Low: two `rand` findings for custom logger unsoundness in `src/backend/Cargo.lock`.

## Post-change alert query status

Re-ran GitHub alert queries after implementation:

- `gh api repos/Scetrov/void-eid/dependabot/alerts --paginate -f state=open` returned `404 Not Found`.
- `gh api repos/Scetrov/void-eid/code-scanning/alerts --paginate -f state=open` returned `404 Not Found`.

Because the API endpoints were unavailable to the active token, comparison was performed with local verification:

- Frontend `npm audit --audit-level=low` reports `found 0 vulnerabilities`.
- Workflow pinning check reports `mutable external uses: 0`.
- Workflow `run` expression check reports `run expressions: 0` for direct `${{ ... }}` interpolation inside shell bodies.
- `aws-lc-sys` resolves to `0.41.0`, satisfying the required `>=0.39.0` floor.
- `rand` vulnerable instances were updated to patched versions where compatible; remaining `lru` and `mysten-metrics` issues are documented below as Sui upstream-blocked.

## Rust dependency remediation

### Remediated

- `aws-lc-sys`: updated through `cargo update -p aws-lc-rs`, moving `aws-lc-rs` from `1.15.4` to `1.17.0` and `aws-lc-sys` from `0.37.1` to `0.41.0` (`>= 0.39.0` required by the change).
- `rand`: updated compatible transitive versions with `cargo update`, moving `rand 0.8.5 -> 0.8.6` and `rand 0.9.2 -> 0.9.4`. `rand 0.10.1` was already patched and `rand 0.7.3` is outside the affected range per `cargo audit` (`unaffected <0.7.0`, patched `>=0.8.6`, `>=0.9.3`, `>=0.10.1`).

### Upstream-blocked / requires Sui graph update

#### `mysten-metrics` 0.7.0

- **Current resolved version:** `0.7.0`.
- **Required safe version:** no patched safe version is available in the current graph; the package was removed from crates.io for malicious code, so remediation requires removing/replacing it or moving to an upstream Sui revision that no longer depends on it.
- **Dependency path:** `void-eid-backend -> sui-sdk v1.74.0 (https://github.com/MystenLabs/sui#8c1a5dbc) -> Sui internal crates -> mysten-metrics 0.7.0`.
- **Attempted update:** targeted compatible Cargo updates were run for remediable crates; `mysten-metrics` is sourced from the git-pinned Sui dependency graph and cannot be independently updated from crates.io in this repository.
- **Follow-up owner/action:** backend/Sui integration owner should evaluate a newer compatible `MystenLabs/sui` git revision or replacement of the affected Sui dependency path. Treat as unresolved until the Sui graph no longer resolves `mysten-metrics`.
- **Mitigation until resolved:** do not expose Sui telemetry/metrics functionality beyond existing backend behavior; keep the dependency path documented in release notes and track upstream Sui fixes.

#### `lru` 0.10.1 and 0.13.0

- **Current resolved versions:** `0.10.1`, `0.13.0`, and `0.16.4`.
- **Required safe version:** per `cargo audit` / `RUSTSEC-2026-0002`, patched `>=0.16.3`; unaffected `<0.9.0`. Therefore `0.16.4` is safe, while `0.10.1` and `0.13.0` remain vulnerable.
- **Dependency paths:**
  - `lru 0.10.1` is pulled through the git-pinned Sui/anemo network stack (`anemo`, `mysten-network`, `sui-sdk v1.74.0` at `MystenLabs/sui#8c1a5dbc`).
  - `lru 0.13.0` is pulled through Sui Move runtime crates (`move-vm-runtime`, `sui-execution`, `sui-rpc-api`, `sui-sdk v1.74.0` at `MystenLabs/sui#8c1a5dbc`).
- **Attempted update:** `cargo update -p lru@0.10.1 -p lru@0.13.0 -p lru@0.16.4` left the vulnerable versions unchanged because upstream Sui crates constrain those major/minor versions.
- **Follow-up owner/action:** backend/Sui integration owner should move to a Sui revision whose `lru` dependencies are `>=0.16.3` or patched internally, or remove the affected Sui dependency paths if no compatible upstream fix is available.
- **Mitigation until resolved:** do not call or expose Sui functionality that would rely on mutating `lru::IterMut` behavior directly from application code; monitor upstream Sui dependency updates.
