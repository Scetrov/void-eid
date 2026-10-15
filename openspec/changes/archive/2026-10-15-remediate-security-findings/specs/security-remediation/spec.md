# security-remediation Delta

## ADDED Requirements

### Requirement: Discord OAuth callback validates the upstream user response
The backend SHALL require an HTTP-success Discord `/users/@me` response with a string `id` containing 1–32 ASCII digits, using the existing `validate_discord_id` rules. Rejected responses SHALL NOT cause user creation or updates, JWT generation, or auth-code issuance. Production OAuth endpoints SHALL remain fixed; regression tests SHALL use an isolated local OAuth boundary without real Discord requests.

#### Scenario: Discord user endpoint returns an error status
- **WHEN** token exchange succeeds but `/users/@me` returns a non-success status
- **THEN** the callback SHALL return generic `400 "Discord OAuth failed"`, log failure category/status without credentials or full upstream bodies, and perform no user mutation or credential issuance

#### Scenario: Discord user endpoint returns an invalid user ID
- **WHEN** the payload's `id` is missing, null, not a string, empty, whitespace, non-digit, or longer than 32 characters
- **THEN** the callback SHALL return generic `400 "Discord OAuth failed"` without user mutation or credential issuance

#### Scenario: Discord user response cannot be retrieved or parsed
- **WHEN** the user request fails at the transport layer or its body is malformed JSON
- **THEN** the callback SHALL retain a generic error response and perform no user mutation or credential issuance

#### Scenario: Valid Discord login succeeds
- **WHEN** token exchange and user retrieval succeed, the ID is valid, the OAuth state is valid, and the identity is not denylisted
- **THEN** the callback SHALL preserve the existing successful login behavior and produce an exchangeable auth code for a valid JWT identifying the expected user

### Requirement: Member notes are scoped to the target user's tribe membership
The backend SHALL verify current target membership in the selected tribe before creating or reading member notes, in addition to existing caller authorization. It SHALL retain tribe-scoped note queries and return `403 Forbidden` for an existing target outside that tribe. This requirement covers create/read only; note editing remains authorship-based and outside this change.

#### Scenario: Note created for a member outside the admin's tribe
- **WHEN** an authorized tribe admin POSTs a note for an existing user who is not in the selected tribe
- **THEN** the backend SHALL return `403 Forbidden` and SHALL NOT insert a note

#### Scenario: Notes read for a member outside the admin's tribe
- **WHEN** an authorized tribe admin GETs notes for an existing user outside the selected tribe
- **THEN** the backend SHALL return `403 Forbidden`

#### Scenario: Note access for a member inside the admin's tribe
- **WHEN** an authorized tribe admin creates or reads notes for a member of the selected tribe
- **THEN** the request SHALL succeed as before and reads SHALL expose only notes belonging to the selected tribe

#### Scenario: Target leaves the tribe after a note was created
- **WHEN** a note was created legitimately and its target subsequently leaves the selected tribe
- **THEN** the note SHALL remain stored but GET access through that tribe SHALL return `403 Forbidden` until target membership is restored

#### Scenario: Caller or target validation fails
- **WHEN** the caller is unauthorized, the target does not exist, or target-membership lookup fails
- **THEN** the backend SHALL preserve caller-authorization rejection, unknown-target 404, or safe 500 handling respectively and SHALL NOT insert notes

### Requirement: Admin roster reads exclude soft-deleted wallets
The backend SHALL exclude wallets with `deleted_at` set from the wallet collections in tribe-roster and member-detail responses while preserving active wallets and existing response shapes. This requirement does not scrub historical audit text.

#### Scenario: Roster contains a member with active and soft-deleted wallets
- **WHEN** an admin fetches a tribe roster or member details for a member with both active and soft-deleted wallets
- **THEN** the wallet collection SHALL contain the active wallets and SHALL NOT contain the soft-deleted wallets, consistent with `GET /api/me`

### Requirement: Dev-only API binaries require explicit stub mode
The `stub_api` binary SHALL start only when the effective `STUB_API` environment value is exactly `1`. It SHALL otherwise fail with a clear non-zero exit before database initialization, seeding, or network binding. This opt-in is an accidental-startup safeguard, not authorization to deploy the binary or protection against credential reuse with overlapping secrets.

#### Scenario: Stub API starts without an accepted opt-in value
- **WHEN** the effective `STUB_API` value is unset, empty, `0`, or any value other than `1`
- **THEN** startup SHALL fail before database initialization or network binding

#### Scenario: Stub API starts with explicit opt-in
- **WHEN** the effective `STUB_API` value is exactly `1` and other startup prerequisites are satisfied
- **THEN** startup SHALL proceed with the existing in-memory test database and reach the readiness endpoint

#### Scenario: E2E test harness launches the stub API
- **WHEN** local or CI e2e tooling launches the stub through Playwright
- **THEN** the shared stub `webServer.env` SHALL supply `STUB_API=1`
