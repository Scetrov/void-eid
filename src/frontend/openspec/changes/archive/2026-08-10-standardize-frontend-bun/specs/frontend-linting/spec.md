## MODIFIED Requirements

### Requirement: Frontend linting runs through Oxlint

The frontend SHALL provide a committed Oxlint configuration and SHALL use Oxlint as the implementation of the `bun run lint` command.

#### Scenario: Lint command succeeds on the repository
- **WHEN** a contributor runs `bun run lint` from the frontend package
- **THEN** Oxlint checks the configured frontend source files and exits successfully when no configured violations exist

#### Scenario: Generated output is excluded
- **WHEN** the frontend contains generated files under `dist`
- **THEN** the lint command excludes those files from analysis

### Requirement: Lint migration remains reproducible

The frontend SHALL declare a released, lockfile-resolved Oxlint dependency compatible with the repository's TypeScript version, and SHALL remove ESLint-only dependencies and configuration after migration validation succeeds.

#### Scenario: Clean dependency installation supports linting
- **WHEN** dependencies are installed from the package manifest and lockfile in a clean checkout
- **THEN** `bun run lint` invokes the declared Oxlint release without requiring a globally installed executable

#### Scenario: ESLint is no longer the active runner
- **WHEN** a contributor inspects the frontend lint script and configuration
- **THEN** no ESLint configuration or ESLint-only package is required to run the lint command
