## Purpose

Define reproducible Bun-based dependency management for the frontend.

## Requirements

### Requirement: Bun is the canonical frontend package manager

The repository SHALL declare and use a single pinned Bun release for frontend dependency installation and package-script execution in active automation, development containers, and contributor documentation.

#### Scenario: Contributor identifies the package manager
- **WHEN** a contributor inspects the frontend package manifest and repository documentation
- **THEN** Bun and its pinned version are identified as the supported frontend package manager

#### Scenario: Automated workflows execute frontend tooling
- **WHEN** CI, container builds, development-container setup, or repository scripts install dependencies or execute package binaries
- **THEN** they use Bun rather than npm, npx, pnpm, or Yarn

### Requirement: Frontend dependency installation is reproducible

The repository SHALL commit `bun.lock` as the only frontend dependency lockfile and SHALL enforce frozen-lockfile installation in automated and containerized workflows.

#### Scenario: Clean automated dependency installation
- **WHEN** frontend dependencies are installed in a clean automated environment
- **THEN** `bun install --frozen-lockfile` succeeds without modifying `bun.lock`

#### Scenario: Manifest and lockfile diverge
- **WHEN** `package.json` contains dependency changes that are not represented in `bun.lock`
- **THEN** frozen dependency installation fails rather than resolving an uncommitted dependency graph

### Requirement: Dependency automation updates the Bun lockfile

Dependabot SHALL use its Bun ecosystem for the frontend dependency directory so dependency update pull requests include compatible updates to both `package.json` and `bun.lock`.

#### Scenario: Dependabot proposes a frontend dependency update
- **WHEN** Dependabot changes a dependency declaration in the frontend package manifest
- **THEN** the same pull request updates `bun.lock` to a graph accepted by frozen Bun installation
