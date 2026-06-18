## ADDED Requirements

### Requirement: Workflow command inputs are safe from template injection
The system SHALL ensure GitHub Actions workflow values derived from inputs, matrix values, event context, or other expressions are not directly interpolated into shell command text when those values are consumed by `run` steps.

#### Scenario: Release version is patched in workflow
- **WHEN** a workflow patches backend or frontend version files from a release or workflow input value
- **THEN** the value SHALL be passed through an environment variable and validated before use in shell commands

#### Scenario: Matrix value is used by a test command
- **WHEN** a workflow uses a matrix value to choose a Playwright project or similar command argument
- **THEN** the command SHALL consume a quoted environment variable instead of embedding the expression directly in `run`

### Requirement: External GitHub Actions are pinned immutably
The system SHALL pin every external GitHub Actions `uses:` reference to a full commit SHA so workflow execution does not depend on mutable tags.

#### Scenario: Workflow references an external action
- **WHEN** CI, release, or code-scanning workflows reference an action from a repository outside the current repository
- **THEN** the `uses:` reference SHALL identify a full commit SHA rather than a version tag or branch name

#### Scenario: Workflow references a local reusable workflow
- **WHEN** a workflow calls a local reusable workflow by path
- **THEN** the local path reference MAY remain unpinned because it is resolved from the current repository checkout

### Requirement: Vulnerable dependency alerts are remediated or documented
The system SHALL update vulnerable frontend and backend dependencies reported by Aikido and GitHub Dependabot to patched versions when compatible patches are available, and SHALL document any alert that is blocked by an upstream dependency constraint.

#### Scenario: Patched dependency version is available
- **WHEN** a vulnerable direct or transitive dependency has a compatible patched version
- **THEN** the package manifest or lockfile SHALL be updated so the resolved version is outside the vulnerable range

#### Scenario: Vulnerability is blocked by upstream dependency graph
- **WHEN** a vulnerable dependency cannot be updated without an incompatible upstream package or unreleased upstream fix
- **THEN** the remediation SHALL record the dependency path, attempted update path, current resolved version, required safe version, and mitigation or follow-up action

#### Scenario: Frontend package managers disagree on resolved versions
- **WHEN** frontend package manifests and lockfiles resolve security-sensitive packages differently
- **THEN** the lockfiles SHALL be regenerated or aligned so the intended safe direct dependency version is consistently resolved

### Requirement: Runtime containers are hardened
The system SHALL configure runtime Docker images to minimize package install surface and run service processes without root privileges wherever the service can operate correctly.

#### Scenario: Runtime package dependencies are installed
- **WHEN** a Dockerfile installs runtime operating-system packages
- **THEN** it SHALL use non-interactive minimal install options, avoid recommended packages unless required, and remove package-manager lists after installation

#### Scenario: Service process starts in a container
- **WHEN** a backend, frontend, or Murmur service container starts its long-running process
- **THEN** the process SHALL run as a non-root user unless a documented package limitation requires root during initialization

#### Scenario: Runtime image exposes a service port
- **WHEN** a runtime Docker image exposes a network service
- **THEN** it SHOULD define a health check that verifies the service is reachable or the process is responsive

### Requirement: Backend security configuration fails safely
The backend SHALL validate security-sensitive configuration at startup and fail fast when required secrets, URLs, or auth settings are invalid.

#### Scenario: Required secret is missing or weak
- **WHEN** the backend starts without a required secret or with a secret that does not meet minimum strength requirements
- **THEN** startup SHALL fail before accepting requests

#### Scenario: Configured frontend origin is invalid
- **WHEN** the backend starts with an invalid frontend origin or CORS origin value
- **THEN** startup SHALL fail instead of silently dropping the invalid origin

### Requirement: Backend authentication checks are hardened
The backend SHALL perform authentication and internal authorization checks using explicit validation rules and comparison methods appropriate for secrets.

#### Scenario: JWT is decoded for protected routes
- **WHEN** the backend validates a JWT for a protected route
- **THEN** it SHALL require the expected signing algorithm and expiration validation, and SHOULD enforce issuer or audience when configured

#### Scenario: Internal shared secret is checked
- **WHEN** the backend validates an internal shared secret header
- **THEN** it SHALL compare the provided value with the configured secret using a constant-time comparison method

### Requirement: Backend request handling limits unsafe inputs and abuse
The backend SHALL apply targeted rate limits and request input constraints to security-sensitive or write-oriented routes.

#### Scenario: Security-sensitive route receives repeated requests
- **WHEN** auth, internal verification, admin mutation, wallet-link, note-write, or similar security-sensitive routes receive repeated requests
- **THEN** the backend SHALL apply rate limits appropriate to the route sensitivity

#### Scenario: Request contains user-controlled strings
- **WHEN** the backend accepts user-controlled JSON fields such as usernames, network identifiers, note content, admin fields, or passwords
- **THEN** it SHALL enforce documented length and format constraints before using or storing those values

### Requirement: Backend client errors do not disclose internals
The backend SHALL avoid returning raw database, dependency, or internal error details in client-facing responses.

#### Scenario: Internal operation fails
- **WHEN** a database query, external dependency call, or internal operation fails while handling a request
- **THEN** the client response SHALL contain a generic safe error message while detailed diagnostics are limited to server-side logs
