## 1. Workflow Supply-Chain Hardening

- [x] 1.1 Inventory every external `uses:` reference in `.github/workflows/ci.yml`, `.github/workflows/release.yml`, and `.github/workflows/codeql.yml` and resolve the current tag to a full commit SHA.
- [x] 1.2 Replace external action tag references with full commit SHAs while leaving local reusable workflow path references unchanged.
- [x] 1.3 Move `inputs.version`, `needs.update-repo-version.outputs.version`, and matrix-derived command arguments from direct `run` interpolation into quoted environment variables.
- [x] 1.4 Validate release/version environment values before modifying `Cargo.toml` or `package.json` in workflows.
- [x] 1.5 Add or run a workflow grep check that confirms no external `uses:` reference remains pinned only to a mutable tag or branch.

## 2. Dependency Vulnerability Remediation

- [x] 2.1 Re-run `gh` Dependabot alert queries and record the open alert list before dependency changes.
- [x] 2.2 Update frontend locks so `zod`, `@babel/core`, `esbuild`, and related transitive packages resolve to non-vulnerable versions consistently across `bun.lock` and `package-lock.json`.
- [x] 2.3 Update backend Cargo locks and dependency constraints to resolve `aws-lc-sys >= 0.39.0` when compatible with the current Sui dependency graph.
- [x] 2.4 Investigate `mysten-metrics`, `rand`, and `lru` alert dependency paths with `cargo tree -i` and update compatible transitive packages where possible.
- [x] 2.5 Document any remaining upstream-blocked Rust alert with dependency path, attempted update, required safe version, current resolved version, and follow-up owner/action.

## 3. Docker Runtime Hardening

- [x] 3.1 Update runtime Docker package installs to use minimal install options and clean package-manager state consistently.
- [x] 3.2 Harden `src/murmur/Dockerfile` so the long-running service executes as `mumble-server` or document why privilege drop is not feasible.
- [x] 3.3 Add health checks for backend, frontend, and Murmur runtime images where reliable service probes are available.
- [x] 3.4 Verify backend, frontend, release frontend, and Murmur Docker images build after hardening changes.

## 4. Backend AppSec Hardening

- [x] 4.1 Add startup validation for required secrets, JWT secret strength, internal shared secret strength, and configured frontend/CORS origins.
- [x] 4.2 Make JWT validation explicit about accepted signing algorithm and expiration, and enforce issuer/audience when configured without breaking existing deployments.
- [x] 4.3 Replace direct internal shared-secret equality checks with constant-time comparison.
- [x] 4.4 Add targeted rate limits to internal verification, admin mutation, wallet-link, note-write, and other security-sensitive routes not already covered.
- [x] 4.5 Add length and format validation for user-controlled request fields in auth, wallet, Mumble, admin, and notes handlers.
- [x] 4.6 Replace raw database/dependency/internal error responses with generic client-safe messages while preserving server-side diagnostics.

## 5. Verification

- [x] 5.1 Run backend formatting, clippy, build, and tests from `src/backend`.
- [x] 5.2 Run frontend dependency install, lint, and build from `src/frontend`.
- [x] 5.3 Run targeted security checks: workflow pinning grep, workflow expression grep for dangerous `run` interpolation, `cargo tree -i` for remediated Rust alerts, and lockfile checks for frontend vulnerable packages.
- [x] 5.4 Re-run `gh` Dependabot and code-scanning alert queries and compare results against the pre-change alert list.
- [x] 5.5 Update the change notes or task output with any unresolved upstream-blocked alert and its documented mitigation.
