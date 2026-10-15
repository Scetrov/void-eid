## 1. Discord OAuth callback validation and test seam

- [x] 1.1 Add a private callback implementation accepting the HTTP client, endpoint URLs, and OAuth configuration; keep the public handler and fixed production Discord endpoints unchanged, with no production endpoint override
- [x] 1.2 Reject non-success `/users/@me` statuses with generic `400 "Discord OAuth failed"` before parsing; log category/status without credentials or full upstream bodies
- [x] 1.3 Require a string ID passing `validate_discord_id` before identity hashing, user lookup/mutation, auditing, JWT generation, or auth-code issuance; return generic 400 for invalid IDs and preserve generic transport/malformed-JSON failures
- [x] 1.4 Add local mock-server callback tests using isolated configuration and valid OAuth state: non-success status, missing/null/wrong-type/empty/whitespace/non-digit/overlength ID, malformed JSON, and transport failure all leave user rows unchanged and auth-code storage empty; no real Discord requests or new dependencies
- [x] 1.5 Test successful callback and code exchange, asserting a valid JWT for the expected user and preserving existing login behavior

## 2. Target-membership enforcement for member notes

- [x] 2.1 Check target membership via `get_user_tribes` in `create_note` after caller authorization and target lookup; reject existing out-of-tribe targets with 403 before insertion and propagate lookup failures safely
- [x] 2.2 Apply the same membership check in `get_notes`, retaining the `(target_user_id, tribe)` query filter and existing unknown-target 404 behavior
- [x] 2.3 Add handler tests for out-of-tribe creation (403 and no insertion), out-of-tribe reads (403), and authorized in-tribe create/read; use valid numeric Discord IDs
- [x] 2.4 Test that reads for a multi-tribe target expose only the selected tribe's notes, and that historical notes remain stored but become unreadable after target membership loss and readable again after restoration; preserve caller-authorization and unknown-target behavior

## 3. Soft-deleted wallet exclusion

- [x] 3.1 Add `AND w.deleted_at IS NULL` to the batch wallet fetch in `get_roster`
- [x] 3.2 Add the same filter to the wallet fetch in `get_roster_member`
- [x] 3.3 Add handler tests with active and soft-deleted wallets, asserting both responses omit deleted wallets while preserving active wallets and response shape; do not alter historical audit text

## 4. Exact-value stub opt-in and tooling

- [x] 4.1 After dotenv loading, require exactly `STUB_API=1` in `stub_api::main`; otherwise exit clearly and non-zero before database initialization, seeding, or listener binding
- [x] 4.2 Supply `STUB_API: '1'` in the stub-only `webServer.env` in `src/frontend/playwright.config.ts`, covering local and CI launches
- [x] 4.3 Add subprocess startup checks in a controlled environment and temporary working directory: unset, empty, `0`, and another invalid value exit before listening; `1` reaches readiness; clean up the positive-case process reliably
- [x] 4.4 Verify the local e2e script and CI Playwright job share this configuration; update any additional launch path only if discovered

## 5. Verification and disposition evidence

- [x] 5.1 Run the backend test suite (`cargo test` in `src/backend`) and confirm all new and existing tests pass
- [x] 5.2 Run the local e2e entrypoint (`scripts/test-e2e.sh`) and the CI-equivalent Playwright browser matrix; record results and resolve failures before closeout
- [x] 5.3 Create `docs/security/security-finding-dispositions.md` mapping all seven findings to source evidence (commit and file/function references), shared root causes, remediation or production false-positive rationale, and verified production packaging; distinguish tribe-scoped notes about non-members from disclosure of another tribe's notes
- [x] 5.4 In the disposition document record historical-note membership-loss behavior, unchanged authorship-based editing, stub in-memory/seeded-user limits, and the fact that opt-in does not prevent token reuse with overlapping secrets; do not claim scanner findings were closed without confirmation

## 6. Archive, final gates, and signed delivery

- [x] 6.1 Validate the OpenSpec change and confirm implementation tasks and verification are complete; archive it before the final commit and PR, following the archive workflow and synchronizing the delta spec (do not archive while implementation is incomplete)
- [ ] 6.2 Run `pre-commit run --all-files` after archival and before the final commit; prompt the user to `pre-commit install` if the hook is absent, and resolve all failures
- [ ] 6.3 Create signed conventional commits with `Co-Authored-By` attribution for the actual model and Pi Coding Agent; if signing fails, stop and offer to retry, never disable signing
- [ ] 6.4 Open a PR with the archived spec reference, test evidence, dispositions, historical-note behavior, and rollback security trade-off; verify the CI matrix passes before declaring delivery complete
