# Design: Remediate Security Findings

## Context

The OAuth callback checks the token exchange status but not the Discord user response status. A JSON error object lacking `id` can pass parsing, yield an empty identity through `unwrap_or_default()`, and reach user persistence and JWT issuance. This is conditional: malformed JSON already fails parsing, and transport failures already return errors.

Note create/read authorize the caller's selected tribe but do not check target membership. Reads already constrain `n.target_user_id` and `n.tribe`; the gap is allowing tribe-scoped notes about non-members, not reading another tribe's notes. Member-detail already checks target membership. Both roster wallet queries omit the soft-delete filter used by `get_me`.

The test-only stub binds all interfaces with permissive CORS and authenticates seeded users without normal authentication. It forcibly uses `sqlite::memory:`; unknown IDs fail its database lookup. It is not included in the production backend image. Explicit opt-in is an accidental-startup safeguard, not a production security boundary.

## Goals / Non-Goals

**Goals:**
- Reject unsuccessful Discord user responses and invalid identity payloads before database mutation, JWT generation, or auth-code issuance.
- Test callback failure and success paths without external network access.
- Enforce target membership on note create/read while preserving caller authorization and tribe-scoped queries.
- Exclude soft-deleted wallets from both roster wallet collections.
- Require exactly `STUB_API=1`, verified on negative and positive startup paths.
- Produce evidence-backed finding dispositions and pass backend, e2e, and pre-commit gates.

**Non-Goals:**
- No changes to caller tribe/global-admin semantics, note editing permissions, or production OAuth endpoint configuration.
- No historical-note deletion, schema migration, dependency additions, or audit-text scrubbing.
- No changes to stub routes, bind address, or CORS; no claim that the opt-in flag makes deployment safe.
- No changes to rate limiting or dependency versions. Add only the diagnostics necessary for the new validation failures.

## Decisions

### D1: Check target membership on note create/read
After caller authorization and target lookup, use `get_user_tribes` and require membership in the selected tribe before fetching or inserting notes. Return the member-detail endpoint's 403 message for an existing out-of-tribe target. Preserve unknown-target 404, propagate membership-query errors as safe 500 responses, and retain the note query's tribe predicate.

Alternative: 404 or empty lists would obscure target existence but diverge from member-detail behavior. Use the existing 403 convention.

### D2: Validate the user response before identity processing
Check `/users/@me` HTTP status before parsing. Non-success statuses and missing, null, wrong-type, empty, whitespace, or non-digit IDs return generic `400 "Discord OAuth failed"`. Validate string IDs with the existing `validate_discord_id` rules (1–32 ASCII digits). Validation precedes denylist hashing, user lookup/mutation, auditing, JWT generation, and auth-code storage. Preserve current generic transport and malformed-JSON errors; those paths must also issue no credentials.

Log failure category/status server-side without access tokens or full upstream response bodies. Alternative: 502 for upstream HTTP failures is defensible, but 400 matches the adjacent token-exchange convention.

### D3: Add a private OAuth test seam
Keep the public handler signature and fixed production Discord URLs. Delegate to a private callback implementation that accepts an HTTP client, OAuth endpoint URLs, and the required OAuth configuration. The production wrapper supplies the existing environment-derived configuration and fixed endpoints; tests provide isolated configuration and a local mock server using existing dependencies. Do not introduce a request-controlled or production environment-controlled endpoint override.

Test the callback orchestration, not only a payload validator: seed a valid OAuth state, mock token/user responses, then assert response status, unchanged user rows on failure, and empty auth-code storage. The success path must produce an exchangeable code and valid JWT for the expected user. Avoid concurrent mutation of process-global environment variables; inject configuration for tests or isolate unavoidable environment-sensitive checks in subprocesses.

Alternative: validator-only unit tests are simpler but cannot establish that persistence and issuance are correctly ordered; live Discord tests are non-deterministic and require credentials.

### D4: Filter wallet collections in both roster handlers
Append `AND w.deleted_at IS NULL` to the batch wallet query in `get_roster` and the single-member wallet query in `get_roster_member`. Preserve active wallets, membership selection, grouping, and response shapes. Historical audit text is not a wallet collection and is outside this fix.

Alternative: hard deletion changes the data model unnecessarily; read filtering matches `get_me`.

### D5: Require exact-value stub opt-in
After dotenv loading, but before database initialization, seeding, or listener binding, require the effective `STUB_API` value to equal `1`. Unset, empty, `0`, and any other value fail with a clear non-zero exit. Add `STUB_API: '1'` only to the stub `webServer.env` in `src/frontend/playwright.config.ts`; this covers both local script and CI launches.

Use subprocess startup checks with a controlled environment and temporary working directory so inherited `.env` files cannot mask negative cases. Verify rejected values exit before listening and `1` reaches the existing readiness endpoint; terminate the positive-case process reliably.

Alternative: accepting variable presence would accidentally enable the stub with `0`; detecting CI implicitly would break deliberate local use. The explicit flag does not prevent shared-secret token reuse and must not be presented as deployment authorization.

## Risks / Trade-offs

- [Historical notes become unavailable when a target leaves a tribe] → retain data and document the current-membership policy; reads resume if membership returns. Such notes may have been created legitimately before departure.
- [Former members/admins can still edit their own notes] → editing remains authorship-based outside this change; do not claim comprehensive note lifecycle containment. Record this boundary for a separate authorization-policy review.
- [Stricter validation rejects transient Discord failures] → users retry login; failures produce safe responses rather than empty-identity credentials.
- [Callback refactoring changes successful logins] → mock the full successful callback and exchange flow as well as negative paths.
- [E2E launch breaks] → add opt-in in the shared Playwright launch configuration and run the local e2e entrypoint and CI matrix.
- [Stub credentials are reusable if deployment secrets overlap] → flag is defense-in-depth only; retain test-only packaging and document secret separation in dispositions.
- [Rollback restores the gaps] → redeploying the previous image is technically possible but should be an explicitly accepted temporary security regression.

## Migration Plan

1. Deploy backend behavior changes with Playwright launch configuration and regression tests in the same change; no database migration is required.
2. Document historical-note visibility and the unchanged editing policy in `docs/security/security-finding-dispositions.md`.
3. Validate the change, run backend/e2e checks, prepare disposition evidence, and archive OpenSpec before the final signed commit and PR. Run pre-commit again after archival so final content is checked.
4. Rollback via the previous image only if necessary, acknowledging that it restores the corrected vulnerabilities.

## Open Questions

None blocking implementation. A broader membership-loss policy for note editing is explicitly deferred rather than silently changed here.
