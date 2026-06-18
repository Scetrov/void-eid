## Context

The repository currently has security findings across multiple layers:

- GitHub Actions workflows contain expression-derived values in shell commands and use mutable action tags for most external actions.
- GitHub Dependabot reports open alerts for Rust and npm packages, including high-severity `aws-lc-sys`, a critical `mysten-metrics` alert from the Sui git dependency tree, and lower-severity `rand`, `lru`, and `@babel/core` alerts.
- Frontend lockfiles are inconsistent for `zod`: `package.json` requests `^4.4.3`, `package-lock.json` resolves `4.4.3`, while `bun.lock` still resolves the direct dependency to `4.3.6`.
- Runtime Dockerfiles already use non-root users for backend and frontend, but the Murmur image and health-check posture need hardening.
- Backend code has security improvement opportunities around configuration validation, JWT validation, internal shared-secret handling, route rate limiting, input validation, and error disclosure.

The change should reduce risk without changing product behavior or creating broad architectural churn.

## Goals / Non-Goals

**Goals:**

- Eliminate workflow template-injection patterns by removing direct GitHub expression interpolation from shell execution paths.
- Pin all external GitHub Actions to full commit SHAs while retaining readable comments or documentation for their source versions where helpful.
- Update vulnerable dependency lock entries and direct version constraints so AppSec and Dependabot findings are resolved or explicitly documented if blocked by upstream Sui git dependencies.
- Harden runtime Docker images with smaller install surfaces, non-root execution where feasible, and service health checks.
- Add targeted backend hardening that preserves existing API behavior while improving startup validation, authentication robustness, and request safety.
- Verify the remediation with existing CI checks plus security-specific dependency and workflow checks.

**Non-Goals:**

- Replacing the Sui SDK or redesigning the wallet/auth architecture.
- Changing public API contracts or user-facing authentication flows.
- Introducing a new container base-image strategy beyond targeted hardening and optional digest pinning.
- Fully eliminating every transitive vulnerability if the only available fix requires an upstream Sui release not yet compatible with this project; those cases should be documented with the dependency path and mitigation.

## Decisions

### Move expression-derived workflow values into environment variables

Workflow values such as release versions and matrix entries will be passed through `env:` and consumed as quoted shell variables. Version values will be validated with a semantic-version regex before use.

- **Rationale:** GitHub expressions in `run:` blocks are expanded before shell execution and can create template-injection risk when values are attacker-controlled or derived from event data.
- **Alternative considered:** Keep interpolation and rely on regex validation after assignment. This still places the expanded value in shell syntax before validation and is weaker.

### Pin actions to immutable SHAs

Every external `uses:` reference in `.github/workflows/*.yml` will be updated from mutable tags to full commit SHAs. Local reusable workflow references such as `./.github/workflows/ci.yml` remain local paths.

- **Rationale:** SHA pinning reduces supply-chain risk from retagging or compromised action releases.
- **Alternative considered:** Pin only third-party actions and leave official `actions/*` mutable. Aikido reports all third-party pinning, but pinning all external actions is more consistent and defensible.

### Prefer lockfile updates over broad dependency replacement

Dependency remediation should start with lockfile regeneration and targeted updates (`cargo update -p ...`, Bun/npm lock refresh). Direct constraints should only change when needed to force safe versions or align package managers.

- **Rationale:** This minimizes blast radius while resolving known vulnerabilities.
- **Alternative considered:** Major dependency upgrades across Sui, Vite, and backend crates. That may be necessary for blocked transitive alerts but should be driven by dependency tree evidence.

### Treat upstream-blocked vulnerabilities as documented exceptions

If `mysten-metrics` or `aws-lc-sys` cannot be remediated without an incompatible Sui git revision, record the dependency path, attempted update, current upstream status, and mitigation in the task output and/or comments.

- **Rationale:** Some alerts originate from the git-pinned Sui dependency graph and may not have a crates.io patch path.
- **Alternative considered:** Remove Sui dependencies immediately. That is out of scope and would change core functionality.

### Apply backend hardening incrementally

Backend security changes should be small and testable: startup configuration validation, explicit JWT validation settings, constant-time internal-secret comparison, added route rate limits, request field caps, and generic client errors.

- **Rationale:** These are AppSec improvements that do not require public API redesign.
- **Alternative considered:** Centralize all validation/auth in a new framework-level abstraction. That adds complexity beyond the remediation need.

### Harden Dockerfiles without breaking runtime ownership

Runtime images should keep or add non-root users, use `--no-install-recommends`, clean package lists, add health checks, and avoid privileged ports. Murmur should run as `mumble-server` if `start.sh` and data-directory permissions support it.

- **Rationale:** Reduces container runtime exposure while preserving service packaging.
- **Alternative considered:** Switch all runtime images to distroless. That would require more compatibility testing for nginx, Murmur, SQLite/OpenSSL, and shell entrypoints.

## Risks / Trade-offs

- **Action SHA pinning reduces automatic action updates** → Track source tag/version comments and periodically refresh SHAs with a maintenance task.
- **Dependency updates may change transitive behavior** → Run backend tests, frontend build/lint, Playwright tests where feasible, and inspect Cargo/Bun lock diffs.
- **Sui git dependencies may block full remediation** → Document dependency paths and either update to a safe Sui revision or record an explicit upstream-blocked exception.
- **Running Murmur as non-root may break package assumptions** → Verify startup and data directory permissions; if root is required for initialization, drop privileges before the long-running process where possible.
- **Stronger JWT/audience/issuer validation may reject existing tokens** → Treat existing tokens as invalid after deployment if claims change; keep claim changes minimal unless issuer/audience are already configured.
- **Generic error responses can reduce debugging detail** → Log internal details server-side while returning stable safe messages to clients.

## Migration Plan

1. Update workflow expressions and pin action SHAs.
2. Update dependency locks and direct constraints; identify any upstream-blocked alerts.
3. Harden Dockerfiles and validate image builds.
4. Apply backend hardening in targeted patches with tests.
5. Run verification: workflow lint/grep checks, `gh` alert review, Cargo tests, frontend lint/build, and Docker builds where practical.
6. Deploy through normal CI/release flow.

Rollback is file-level: revert workflow, lockfile, Dockerfile, or backend hardening commits independently if a specific change breaks CI or runtime behavior.

## Open Questions

- Which exact Sui git revision, if any, resolves the `mysten-metrics` and `aws-lc-sys` paths without breaking backend compilation?
- Should official GitHub-owned actions also be pinned to SHAs, or only non-GitHub third-party actions? This design recommends pinning all external actions for consistency.
- Should JWT issuer/audience be configured through new environment variables immediately, or should this change only enforce algorithm and secret-strength validation to avoid token migration?
