# Remediate Security Findings

## Why

An AI-assisted scan reported seven High-severity findings on `main`. Review identified a Discord OAuth response-validation flaw and a missing target-membership check shared by note create/read; it did not establish disclosure of another tribe's notes. This change addresses those gaps and adds optional wallet-read and test-binary safeguards.

## What Changes

- `GET /api/auth/discord/callback`: require a successful `/users/@me` response and a string Discord ID satisfying the existing `validate_discord_id` rules before any user mutation or credential issuance. Add a private mockable OAuth boundary without making production endpoints user-configurable.
- `POST` and `GET /api/roster/{discord_id}/notes`: require the target to belong to the selected tribe, matching member-detail authorization. Existing reads already filter notes by both target and authorized tribe; this closes missing target-membership enforcement, not an established bypass of the note's tribe filter.
- `GET /api/roster` and `GET /api/roster/{discord_id}`: exclude soft-deleted wallets from their wallet collections, consistent with `GET /api/me`.
- `stub_api`: require exactly `STUB_API=1` before initialization/listener binding to reduce accidental startup. The stub already forces an in-memory database and only issues credentials for existing seeded users. The flag does not make it production-safe or prevent credential reuse if secrets overlap.
- Supply `STUB_API=1` through the shared Playwright stub `webServer.env`, covering local and CI e2e runs.
- Record evidence-backed scan dispositions in `docs/security/security-finding-dispositions.md`, including the shared root cause of the note findings and the production exclusions for the stub.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `security-remediation`: add requirements for Discord user-response validation, target membership on note create/read, soft-deleted wallet exclusion, and exact-value test-binary opt-in.

## Impact

- Backend: `src/backend/src/auth.rs`, `notes.rs`, `roster.rs`, and `bin/stub_api.rs`; tests exercise the real handlers with local fixtures.
- Tooling: `src/frontend/playwright.config.ts`; verify `scripts/test-e2e.sh` and `.github/workflows/ci.yml` work through this shared launch configuration. Separate flag settings are unnecessary unless another launch path is discovered.
- API: non-success user responses and invalid IDs return generic 400 errors; valid logins retain the existing flow. Existing unknown-target 404 and caller-authorization behavior remain unchanged; existing out-of-tribe targets return 403 on note create/read.
- Historical notes about members who leave a tribe remain stored but become unavailable through that tribe's GET endpoint until membership is restored. Note editing remains authorship-based and is outside this change.
- No schema migration, dependency additions, or changes to the authorized admin data model. Soft-delete filtering applies to wallet collections, not historical audit text.
