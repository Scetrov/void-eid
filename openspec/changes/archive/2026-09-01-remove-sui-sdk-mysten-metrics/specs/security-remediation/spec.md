## MODIFIED Requirements

### Requirement: Vulnerable dependency alerts are remediated or documented
The system SHALL update vulnerable frontend and backend dependencies reported by Aikido and GitHub Dependabot to patched versions when compatible patches are available, SHALL remove vulnerable direct or transitive dependencies when equivalent application behavior can be preserved with a narrower implementation, and SHALL document any alert that is blocked by an upstream dependency constraint.

#### Scenario: Patched dependency version is available
- **WHEN** a vulnerable direct or transitive dependency has a compatible patched version
- **THEN** the package manifest or lockfile SHALL be updated so the resolved version is outside the vulnerable range

#### Scenario: Vulnerable transitive dependency is only needed through unused upstream surface
- **WHEN** a vulnerable transitive dependency is pulled in only by an upstream package surface the application does not use
- **THEN** the application SHALL remove or replace that upstream package while preserving the application behavior that previously depended on it

#### Scenario: Vulnerability is blocked by upstream dependency graph
- **WHEN** a vulnerable dependency cannot be updated or removed without an incompatible upstream package, unreleased upstream fix, or functionality reduction
- **THEN** the remediation SHALL record the dependency path, attempted update or removal path, current resolved version, required safe version, and mitigation or follow-up action

#### Scenario: Frontend package managers disagree on resolved versions
- **WHEN** frontend package manifests and lockfiles resolve security-sensitive packages differently
- **THEN** the lockfiles SHALL be regenerated or aligned so the intended safe direct dependency version is consistently resolved
