## Plan: Refactor CI to separate Playwright E2E job with browser matrix (final)

Break the monolithic `frontend` CI job into lint/build and E2E jobs, add a Chromium/Firefox/WebKit browser matrix with `timeout-minutes: 15`, serve production builds via `bun run preview` on port 4173, fix the `stub.spec.ts` user_id mismatch, normalize commands to `bun`, and add VS Code tasks covering both dev and E2E scenarios.

### Steps

1. **Trim the `frontend` job to lint + build only** — In [ci.yml](../workflows/ci.yml), remove all Rust, Playwright, and E2E steps. Keep: checkout → setup Bun → install deps → lint → build. Upload `src/frontend/dist` via `actions/upload-artifact@v4`. 

2. **Add stub_api artifact to the `backend` job** — In [ci.yml](../workflows/ci.yml), append `cargo build --release --bin stub_api` and upload `src/backend/target/release/stub_api` via `actions/upload-artifact@v4`. 

3. **Create an `e2e` job with browser matrix** — New job: `needs: [backend, frontend]`, `timeout-minutes: 15`, `fail-fast: false`, matrix over `project: [chromium, firefox, webkit]`. Each leg downloads both artifacts, starts stub_api on `:5038`, runs `bun run preview` on `:4173`, installs browser via `bun x playwright install --with-deps ${{ matrix.project }}`, runs `bun x playwright test --project=${{ matrix.project }}`, and uploads `blob-report` as `blob-report-${{ matrix.project }}` with `if: ${{ !cancelled() }}`.

4. **Add a `merge-reports` job** — `needs: [e2e]`, `if: ${{ !cancelled() }}`. Downloads all blob reports, runs `bun x playwright merge-reports --reporter html ./all-blob-reports`, uploads merged HTML report with 14-day retention.

5. **Update `playwright.config.ts`** — In [playwright.config.ts](../../src/frontend/playwright.config.ts):
   - Add `firefox` (`devices['Desktop Firefox']`) and `webkit` (`devices['Desktop Safari']`) projects alongside `chromium`.
   - Switch `reporter` to `process.env.CI ? 'blob' : 'html'`.
   - Change `baseURL` to `process.env.BASE_URL || 'http://localhost:4173'`.
   - Make `webServer` conditional: in CI → empty array; locally → `bun run preview` on `:4173` and `cargo run --bin stub_api` on `:5038`.

6. **Normalize E2E test URLs to glob patterns** — Convert all hardcoded `http://localhost:5038/...` in `page.route()` / `page.waitForRequest()`:
   - [home.spec.ts](../../src/frontend/e2e/home.spec.ts) — `page.route(...)` → `'**/api/me'`. 
   - [login.spec.ts](../../src/frontend/e2e/login.spec.ts) — `.includes(...)` → `'/api/auth/discord/login'`. 
   - [roster-member.spec.ts](../../src/frontend/e2e/roster-member.spec.ts) — "back to roster" test routes → `'**/api/me'` and `'**/api/roster/789*'`. 

7. **Fix `stub.spec.ts` user_id and extract API_URL** — In [stub.spec.ts](../../src/frontend/e2e/stub.spec.ts), add `const API_URL = process.env.API_URL || 'http://localhost:5038'` and change `user_id=admin-user-id` to `user_id=1001` to match [stub_api.rs](../../src/backend/src/bin/stub_api.rs#L74). 

8. **Normalize `npm`/`npx` to `bun` across scripts and docs** —
   - [test-e2e.sh](../../scripts/test-e2e.sh#L60) — `npx playwright test` → `bun x playwright test`.
   - [test-e2e.sh](../../scripts/test-e2e.sh#L55) — `./node_modules/.bin/vite` → `bun run vite --`. 
   - [contributing.md](../../docs/contributing.md) — both `npm run lint` references → `bun run lint`. 

9. **Update VS Code tasks for both dev and E2E workflows** — In [tasks.json](../../.vscode/tasks.json), create tasks covering two distinct scenarios:

   **Dev workflow** (day-to-day development with HMR + auto-reload):
   - Update existing "Run Frontend" task: change `type` from `"npm"` to `"shell"`, set `command` to `"bun run dev"` with `cwd` pointing to `src/frontend`. Runs Vite dev server with HMR on `:5173`.
   - Existing "Run Backend" task stays as-is (`cargo watch -x run`). Provides auto-rebuild on file changes.
   - Add compound task **"Dev"** with `dependsOn: ["Run Backend", "Run Frontend"]` and `dependsOrder: "parallel"`. Starts the full dev stack with one command.

   **E2E workflow** (local Playwright testing against production build + stub_api):
   - Add **"Run Stub API"** task: `cargo watch -x 'run --bin stub_api'` with `cwd` at `src/backend` and env vars `DATABASE_URL=sqlite::memory:`, `JWT_SECRET=stub-jwt-secret`, `FRONTEND_URL=http://localhost:4173`.
   - Add **"Run Preview"** task: `bun run preview` with `cwd` at `src/frontend`. Serves the production build on `:4173`.
   - Add compound task **"E2E Dev"** with `dependsOn: ["Run Stub API", "Run Preview"]` and `dependsOrder: "parallel"`. Starts the E2E test stack — then run `bun x playwright test` separately in a terminal, or use the Playwright VS Code extension.