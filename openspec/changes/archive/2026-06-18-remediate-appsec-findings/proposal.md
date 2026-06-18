## Why

Recent Aikido AppSec and GitHub Dependabot findings identify supply-chain, dependency, container, and backend hardening gaps that increase the risk of compromised CI workflows, vulnerable transitive libraries, and avoidable runtime exposure. Addressing them now reduces critical/high security risk before further releases publish artifacts and container images.

## What Changes

- Remediate GitHub Actions template-injection patterns by moving expression-derived values into validated environment variables before shell use.
- Pin all external GitHub Actions in CI, release, and CodeQL workflows to immutable commit SHAs.
- Update vulnerable frontend and backend dependencies/locks reported by Aikido and `gh`, including `aws-lc-sys`, `zod`, `@babel/core`, `rand`, `lru`, and related transitive packages where feasible.
- Harden Docker runtime images by reducing package install surface, enforcing non-root execution where safe, and adding health checks for service containers.
- Improve backend security posture for configuration validation, JWT validation, internal-secret comparison, rate limiting, input validation, and client-facing error handling.
- Add verification steps that make the security remediation repeatable through CI, dependency audit commands, and targeted tests.

## Capabilities

### New Capabilities
- `security-remediation`: Covers CI supply-chain hardening, dependency vulnerability remediation, container runtime hardening, and backend AppSec improvements required by the current security scan findings.

### Modified Capabilities

## Impact

- GitHub workflows: `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `.github/workflows/codeql.yml`.
- Frontend dependencies and locks: `src/frontend/package.json`, `src/frontend/bun.lock`, `src/frontend/package-lock.json`.
- Backend dependencies and locks: `src/backend/Cargo.toml`, `src/backend/Cargo.lock`, and potentially transitive Sui-related dependency revisions.
- Dockerfiles: `src/backend/Dockerfile`, `src/frontend/Dockerfile`, `src/frontend/Dockerfile.release`, `src/murmur/Dockerfile`, and development Dockerfiles if scan-relevant.
- Backend security code: CORS startup validation, JWT validation, secret configuration, internal auth middleware, rate limiting, input validation, and error responses under `src/backend/src/`.
- CI/build verification may update lockfiles and require dependency tree validation through `gh`, Cargo, Bun/npm, and existing test suites.
