## Why

The frontend currently installs dependencies with Bun in CI and container builds while retaining npm configuration, commands, and `package-lock.json`. This split causes Dependabot updates to leave `bun.lock` stale and breaks reproducible `bun install --frozen-lockfile` workflows.

## What Changes

- Declare Bun 1.3.14 as the frontend's canonical package manager.
- Configure Dependabot to update Bun dependencies and `bun.lock`.
- Use Bun consistently in CI, the development container, scripts, Playwright configuration, and contributor documentation.
- Remove `package-lock.json` and prevent it from being reintroduced.
- Require frozen Bun lockfile installs for automated and containerized workflows.

## Capabilities

### New Capabilities
- `frontend-package-management`: Defines Bun as the canonical frontend package manager and requires reproducible dependency installation from `bun.lock`.

### Modified Capabilities
- `frontend-linting`: Changes the documented and supported lint command from npm to Bun.

## Impact

Affected areas include the frontend package manifest and lockfiles, Dependabot configuration, GitHub Actions, the development container, Playwright launch scripts, repository documentation, and frontend linting requirements. No application runtime API changes are introduced.
