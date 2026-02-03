## Plan: Rider Check & Friendly Status System (Final)

Implement rider name lookup with configurable fuzzy matching (admin-selectable algorithm), friendly/hostile status display, and admin-only friendly marking. Uses `moka` in-memory caching with startup warming. Includes comprehensive backend and E2E tests.

### Steps

1. **Create database migrations**: Add [05_add_rider_name.sql](src/rust/migrations/) with:
   - `ALTER TABLE wallets ADD COLUMN rider_name TEXT`
   - `CREATE TABLE friendly_riders (id, wallet_id FK, added_by_user_id FK, created_at)` — admin who marked friendly
   - `CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT, updated_at, updated_by FK)` — for app-wide config
   - Seed test rider names and default setting `fuzzy_algorithm = "jaro_winkler"`

2. **Add dependencies and caching infrastructure**: Add to [Cargo.toml](src/rust/Cargo.toml):
   - `strsim = "0.11.1"` for fuzzy matching
   - `moka = { version = "0.12", features = ["future"] }` for async in-memory cache

   Update [state.rs](src/rust/src/state.rs) to add `settings_cache: Cache<String, String>` with 5-minute TTL.

3. **Build settings module with cache warming**: Create [src/rust/src/settings.rs](src/rust/src/settings.rs):
   - `GET /api/settings` — returns all settings (any authenticated user), reads from cache with DB fallback
   - `PUT /api/settings/:key` — updates setting value (admin-only), invalidates cache, audit logged
   - `pub async fn warm_cache(state: &AppState)` — loads all settings from DB into cache on startup
   - Define `FuzzyAlgorithm` enum: `JaroWinkler`, `NormalizedLevenshtein`

4. **Call cache warming on startup**: Update [main.rs](src/rust/src/main.rs) to call `settings::warm_cache(&state).await` after migrations run, before server starts listening.

5. **Build riders module with configurable algorithm**: Create [src/rust/src/riders.rs](src/rust/src/riders.rs):
   - `GET /api/riders/autocomplete?q=` — loads `fuzzy_algorithm` from cache, scores all `rider_name` values using selected algorithm, returns top 10 with `{ rider_name, tribe, score }`
   - `POST /api/riders/check` — finds rider, returns `{ status, rider_name, tribe }`. Friendly if same tribe OR exists in `friendly_riders.added_by_user_id` matches any user in requester's tribe
   - `GET /api/riders/friendly` — admin-only, lists all friendly riders for requester's tribe
   - `POST /api/riders/friendly` — admin-only, creates entry, audit logged
   - `DELETE /api/riders/friendly/:id` — admin-only, removes entry, audit logged

6. **Add AdminUser extractor and register routes**: Create `AdminUser` extractor in [auth.rs](src/rust/src/auth.rs) that validates `is_admin`. Wire all handlers in [main.rs](src/rust/src/main.rs). Register OpenAPI schemas: `RiderSuggestion`, `RiderCheckRequest`, `RiderCheckResult`, `Setting`, `FriendlyRider`.

7. **Create frontend `/check` route**: Add [src/sui/src/routes/check.tsx](src/sui/src/routes/check.tsx):
   - Protected page (login required, not admin-only)
   - Autocomplete input with debounced `useQuery` (300ms), dropdown showing `"RiderName (TribeName)"`
   - Result card: blue/shield for friendly, red/alert for hostile
   - Angular styling per design system

8. **Add "Check Rider" to main nav**: Update [DashboardLayout.tsx](src/sui/src/components/DashboardLayout.tsx) to include "Check Rider" nav link for all authenticated users (alongside existing Roster link for admins).

9. **Create admin settings page**: Add [src/sui/src/routes/settings/index.tsx](src/sui/src/routes/settings/):
   - Admin-only page following roster pattern
   - Radio/select for `fuzzy_algorithm`: "Jaro-Winkler (better for typos)" vs "Levenshtein (edit distance)"
   - Save button calls `PUT /api/settings/fuzzy_algorithm`

10. **Add friendly management to roster detail**: Extend [roster/$id.tsx](src/sui/src/routes/roster/$id.tsx):
    - For each wallet with `rider_name`, show "Mark Friendly" / "Remove Friendly" toggle button
    - Display current friendly status with visual indicator

11. **Write backend tests**: Add `#[cfg(test)]` modules to:
    - [settings.rs](src/rust/src/settings.rs) — test get/update, admin validation, cache invalidation, `warm_cache()` populates correctly, audit logging
    - [riders.rs](src/rust/src/riders.rs) — test autocomplete with both algorithms, friendly logic (same tribe, cross-tribe marked, hostile), threshold filtering, CRUD permission checks

12. **Write E2E tests**: Add Playwright tests:
    - [e2e/check.spec.ts](src/sui/e2e/check.spec.ts) — test autocomplete interaction, result display for friendly/hostile, unauthenticated redirect
    - [e2e/settings.spec.ts](src/sui/e2e/settings.spec.ts) — test admin access, algorithm toggle persistence, non-admin denied
