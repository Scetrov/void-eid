## Purpose

Define requirements for reproducible Oxlint-based frontend linting that preserves the repository's lint command and policy coverage.

## Requirements

### Requirement: Frontend linting runs through Oxlint

The frontend SHALL provide a committed Oxlint configuration and SHALL use Oxlint as the implementation of the `npm run lint` command.

#### Scenario: Lint command succeeds on the repository
- **WHEN** a contributor runs `npm run lint` from the frontend package
- **THEN** Oxlint checks the configured frontend source files and exits successfully when no configured violations exist

#### Scenario: Generated output is excluded
- **WHEN** the frontend contains generated files under `dist`
- **THEN** the lint command excludes those files from analysis

### Requirement: Lint policy covers supported TypeScript and React checks

The Oxlint configuration SHALL enable supported equivalents for the repository's TypeScript, JavaScript, React Hooks, and React Refresh lint policy, or SHALL document each unsupported rule as an intentional coverage gap with rationale.

#### Scenario: Existing rule policy is reviewed during migration
- **WHEN** the ESLint configuration is compared with the Oxlint configuration
- **THEN** every existing rule is mapped to an Oxlint equivalent, replaced by a documented alternative, or listed as an accepted unsupported check

#### Scenario: Route-specific exception is preserved or deliberately replaced
- **WHEN** linting files under `src/routes`
- **THEN** the route-specific React Refresh behavior is equivalent to the existing policy or the changed behavior is explicitly documented and validated

### Requirement: Lint migration remains reproducible

The frontend SHALL declare a released, lockfile-resolved Oxlint dependency compatible with the repository's TypeScript version, and SHALL remove ESLint-only dependencies and configuration after migration validation succeeds.

#### Scenario: Clean dependency installation supports linting
- **WHEN** dependencies are installed from the package manifest and lockfile in a clean checkout
- **THEN** `npm run lint` invokes the declared Oxlint release without requiring a globally installed executable

#### Scenario: ESLint is no longer the active runner
- **WHEN** a contributor inspects the frontend lint script and configuration
- **THEN** no ESLint configuration or ESLint-only package is required to run the lint command
