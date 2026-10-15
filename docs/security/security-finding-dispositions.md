# Security Finding Dispositions

**Change:** `remediate-security-findings`
**Base commit:** `70551965b49b9f62ec6688d1932487287c5469ff` (point of divergence on `main`)
**Delivery:** this document ships in the change's conventional commit (created after the OpenSpec change is archived); see the PR for the final SHA.

## Provenance and scope

An AI-assisted scan reported **seven High-severity findings** on `main`. The scan's raw report
is not stored in this repository. This document is an **engineering disposition and
verification record**, not a reproduction of the scanner's output. It maps the scan's reported
items to the concrete code locations and root causes this change addresses, records the
remediation (or production false-positive rationale) for each, and cites source and test
evidence.

> **This document does not assert that the scanner re-ran and confirmed the findings closed.**
> A fresh scan is the authoritative confirmation and is out of scope here.

The seven findings group into **four root-cause areas**:

1. Discord OAuth callback response validation — F1, F2
2. Member-note target membership, shared by create and read — F3, F4
3. Admin-roster soft-deleted wallet exclusion, shared by two queries — F5, F6
4. Dev-only stub binary opt-in — F7

## F1 — OAuth callback ignores the `/users/@me` HTTP status (remediated)

- **Location:** `src/backend/src/auth.rs` → `discord_callback` (handler), `perform_discord_oauth` (private implementation).
- **Concern:** the callback verified the token-exchange status but not the `/users/@me` response status; a JSON error object lacking `id` could pass parsing and, via `unwrap_or_default()`, yield an empty identity that reached user persistence and JWT issuance.
- **Root cause:** the upstream user response status was not checked before identity processing; a missing `id` collapsed to an empty string instead of failing closed.
- **Remediation:** `perform_discord_oauth` rejects any non-success `/users/@me` status with generic `400 "Discord OAuth failed"` **before** parsing the body, logging only the failure category/status (no credentials, no full upstream body).
- **Evidence:** `auth.rs::oauth_callback_tests::oauth_user_non_success_is_rejected_before_body_validation`, `oauth_user_malformed_json_is_rejected`; `oauth_rejections_leave_no_user_or_auth_code` (asserts no user row or auth code is written).

## F2 — OAuth callback does not validate the Discord ID before identity processing (remediated)

- **Location:** `src/backend/src/auth.rs` → `perform_discord_oauth`; reuses the existing `validate_discord_id`.
- **Concern:** a missing, null, non-string, empty, whitespace, non-digit, or over-length `id` could proceed to identity hashing, user lookup/mutation, auditing, JWT generation, and auth-code issuance.
- **Root cause:** the string `id` was trusted from the upstream payload without the existing 1–32 ASCII-digit rule being applied before any mutation or credential issuance.
- **Remediation:** a string `id` passing `validate_discord_id` is now required **before** denylist hashing, user lookup/mutation, auditing, JWT generation, or auth-code storage. All invalid-ID shapes return generic 400 and issue no credentials. Transport failures and malformed JSON keep their generic errors and also issue no credentials.
- **Evidence:** `auth.rs::oauth_callback_tests::oauth_invalid_discord_id_is_rejected` (missing/null/wrong-type/empty/whitespace/non-digit/over-length), `oauth_transport_failure_is_rejected`, `oauth_token_non_success_is_rejected`, `oauth_token_malformed_json_is_rejected`, `oauth_missing_access_token_is_rejected`. Success: `oauth_success_returns_validated_profile`, `complete_login_generates_auth_code_and_persists_user`.
- **Production endpoints:** fixed and unchanged. The private implementation takes the client, endpoint URLs, and OAuth configuration as parameters (a test seam) but there is **no request- or environment-controlled production endpoint override** (change design D3).

## F3 — Note creation allows targets outside the selected tribe (remediated)

- **Location:** `src/backend/src/notes.rs` → `create_note`.
- **Concern:** an authorized tribe admin could create a note for an existing user who is not a member of the selected tribe.
- **Root cause (shared with F4):** create/read authorized the **caller's** selected tribe but did not verify the **target's** membership in that tribe.
- **Remediation:** after caller authorization and target lookup, `create_note` checks target membership via `get_user_tribes` and returns `403` for an existing out-of-tribe target **before** insertion; membership-lookup failures propagate as safe errors.
- **Evidence:** `notes.rs::note_create_out_of_tribe_forbidden_no_insert` (asserts 403 **and** no row inserted), `note_create_in_tribe_allowed`.

## F4 — Note reads allow targets outside the selected tribe (remediated)

- **Location:** `src/backend/src/notes.rs` → `get_notes`.
- **Concern:** an authorized tribe admin could read notes for an existing user who is not a member of the selected tribe.
- **Root cause (shared with F3):** missing target-membership enforcement on read.
- **Remediation:** the same `get_user_tribes` membership check is applied in `get_notes`; the `(target_user_id, tribe)` note query predicate is **retained**. An existing out-of-tribe target returns `403`; unknown targets still return `404`; caller-authorization behavior is unchanged.
- **Evidence:** `notes.rs::note_read_out_of_tribe_forbidden`, `note_read_in_tribe_allowed`, `note_read_multi_tribe_exposes_only_selected` (a multi-tribe target exposes only the selected tribe's notes), `note_read_membership_loss_and_restoration` (see behavioral boundary below).

## F5 — `GET /api/roster` includes soft-deleted wallets (remediated)

- **Location:** `src/backend/src/roster.rs` → `get_roster` (batch wallet query).
- **Concern:** the tribe-roster wallet collection returned wallets whose `deleted_at` is set, inconsistent with `GET /api/me`.
- **Root cause (shared with F6):** the roster batch wallet query omitted the soft-delete filter used elsewhere.
- **Remediation:** appended `AND w.deleted_at IS NULL` to the batch wallet query. Active wallets, membership selection, grouping, and response shape are preserved.
- **Evidence:** `roster.rs::test_roster_omits_deleted_wallets`.

## F6 — `GET /api/roster/{discord_id}` includes soft-deleted wallets (remediated)

- **Location:** `src/backend/src/roster.rs` → `get_roster_member` (single-member wallet query).
- **Concern:** the member-detail wallet collection returned soft-deleted wallets.
- **Root cause (shared with F5):** the single-member wallet query omitted the soft-delete filter.
- **Remediation:** appended `AND w.deleted_at IS NULL` to the member wallet query.
- **Evidence:** `roster.rs::test_roster_member_omits_deleted_wallets`.

## F7 — Dev-only stub binary starts without explicit opt-in (safeguard added; not a security boundary)

- **Location:** `src/backend/src/bin/stub_api.rs` → `main`.
- **Concern:** the test-only stub binds all interfaces with permissive CORS and authenticates seeded users without normal authentication, so an accidental startup would be dangerous.
- **Root cause:** there was no gate preventing the binary from starting without an explicit intent to run it.
- **Remediation (safeguard, not a boundary):** after `dotenvy` loading and **before** database initialization, seeding, or listener binding, `main` requires the effective `STUB_API` value to be exactly `1`; any other value (unset, empty, `0`, or any other) exits non-zero with a clear message. `STUB_API: '1'` is supplied only in the stub `webServer.env` in `src/frontend/playwright.config.ts`, which covers both the local `scripts/test-e2e.sh` path and the CI Playwright job.
- **Evidence:** `tests/stub_optin.rs::stub_refuses_without_exact_opt_in` (unset/empty/`0`/other exit before listening), `stub_starts_when_exactly_opted_in` (`1` reaches the readiness endpoint).
- **Production packaging:** the stub is **not included in the production backend image**, forcibly uses `sqlite::memory:`, and issues credentials only for existing seeded users. See "Stub production exclusions" below.

## Shared root causes

- **F3 and F4 (note membership)** share one root cause: note create and read authorized the **caller's** tribe but never verified the **target's** membership in that tribe. Both are fixed by the same `get_user_tribes` membership check applied in each handler.
- **F5 and F6 (soft-deleted wallets)** share one root cause: both roster wallet queries omitted the `deleted_at IS NULL` predicate that `GET /api/me` already applied. Both are fixed by the same predicate appended to each query.

## Tribe-scoped notes about non-members vs. cross-tribe disclosure

The scan **did not establish** disclosure of another tribe's notes. The note query already constrains results by **both** `target_user_id` and the caller's **selected** `tribe`. What was missing — and what this change closes — is allowing **tribe-scoped notes *about* a non-member target**: a note whose `target_user_id` is a user outside the selected tribe. After the fix, an existing out-of-tribe target is rejected with `403` on both create and read.

The distinction matters: reads never cross tribe boundaries (the `tribe` predicate was always present). The gap was **target membership**, which is now enforced. A `403` here means "the target is not a member of the tribe you are viewing" — it does not mean "another tribe's notes were exposed."

## Stub production exclusions (false-positive rationale for F7 in production)

In the production deployment the stub is a non-issue:

- It is **excluded from the production backend image** (only the real backend binary ships).
- It **forces** `sqlite::memory:` (no persistent store; the database is empty on start and gone on exit).
- It **issues credentials only for existing seeded users**; unknown IDs fail the database lookup.
- The `STUB_API=1` opt-in is an **accidental-startup safeguard**, not authorization to deploy the binary and not a production security boundary.

## Behavioral boundaries and residual limits

These are recorded as deliberate, in-scope boundaries (not regressions) and as items to keep
visible to anyone reasoning about note/wallet lifecycle:

- **Historical notes and membership loss.** A note created legitimately while its target was a
  member **remains stored** if the target later leaves the tribe. It becomes **unreadable**
  through that tribe's `GET /api/roster/{id}/notes` (returns `403`) until the target's
  membership in that tribe is restored; reads then resume. No historical note is deleted by
  this change (a non-goal). This is verified by `notes.rs::note_read_membership_loss_and_restoration`.
- **Note editing is unchanged and authorship-based.** This change covers **create/read** only.
  Editing (`PUT`) remains authorized by authorship and is **outside this change**. A former
  member or admin can still edit a note they authored. Do not read this change as comprehensive
  note-lifecycle containment; a broader membership-loss policy for editing is a separate
  authorization review.
- **Stub limits.** The stub is in-memory and serves only seeded users; it is not production-safe
  and must not be treated as such. Its CORS/bind-address behavior is intentionally **unchanged** by
  this change (a non-goal).
- **The opt-in flag does not prevent credential reuse with overlapping secrets.** `STUB_API=1`
  gates startup only. If the stub were run in an environment whose JWT/secret material overlaps
  production, issued tokens could be reused. **Do not present the flag as deployment
  authorization or as a guarantee against secret overlap**; keep test and production secrets
  separate.
- **Scanner confirmation is separate.** Nothing above should be read as the scanner having
  re-verified closure; verification here is by regression tests and e2e (below), not a re-scan.

## Verification evidence

Captured on the delivery branch at the base commit above:

- **Backend suite (`cargo test`, `src/backend`): all tests pass** — 78 unit/integration tests in
  the library and 2 subprocess opt-in tests (`tests/stub_optin.rs`). New coverage: 10 mock-server
  OAuth callback tests (`auth.rs`), 6 note membership tests (`notes.rs`), 2 soft-deleted-wallet
  roster tests (`roster.rs`), 2 stub opt-in tests. The full suite is green.
- **`cargo fmt --check`:** clean.
- **`cargo clippy -- -D warnings`:** clean (no new warnings).
- **E2E:** the local entrypoint and the CI-equivalent Playwright matrix share the stub
  `webServer.env` (`STUB_API=1`). On the local host, **Chromium: 19/19 passed** (full
  harness + stub API + Vite frontend). **Firefox and WebKit could not be executed on this
  host** for environmental reasons unrelated to the change: Firefox's headless compositor
  cannot map a default framebuffer on this displayless host, and the WebKitGTK engine shared
  libraries are not present in the local Nix store. The CI Playwright job installs browser
  dependencies and is the authoritative cross-browser signal; the shared `webServer.env`
  change is browser-independent, so the stub opt-in is exercised identically across all three
  projects.

## Rollback

Rolling back to the previous image is technically trivial (redeploy the prior build) but
**restores the corrected vulnerabilities** (F1–F7). It must be treated as an explicitly accepted,
temporary security regression, not a neutral rollback.
