# Void eID - Copilot Instructions

## IMPORTANT

It is unacceptable to use `git commit --no-verify` or to bypass linting/formatting checks. All code must pass `cargo fmt`, `cargo clippy`, and `bun run lint` before committing. The CI pipeline enforces this, and any commit that fails these checks will be rejected. Equally it is completely unacceptable to disable GPG signing in any way.

## Architecture Overview

**Discord-to-Sui Wallet identity verification** with **tribe-based multi-tenancy** and **Mumble voice integration**.

- **Backend** (`src/backend/`): Rust/Axum API with SQLite — Discord OAuth2, Sui wallet signature verification, Mumble account management
- **Frontend** (`src/frontend/`): React 19/Vite SPA — TanStack Router + Sui dApp Kit
- **Mumble** (`src/murmur/`): Murmur server with Python Ice authenticator that calls backend's internal API

**Core Flow**: Discord OAuth → JWT issued → link Sui wallet (nonce→sign→verify) → wallet associated with tribes → admins view tribe-filtered rosters

**Critical Pattern**: Tribe isolation is enforced at the DB query level — always filter by tribe when fetching rosters/members/notes.

## Development Commands

```bash
# Backend (from src/backend/)
cargo run                    # API at localhost:5038
cargo fmt && cargo clippy    # Required before commits

# Frontend (from src/frontend/)
bun run dev                  # Dev server at localhost:5173
bun run lint                 # ESLint check
bun run test                 # Vitest unit tests

# Full stack
docker compose up            # All services with hot-reload

# Test data (from src/backend/)
python3 scripts/seed_test_data.py    # Seed 100 test users with tribes
python3 scripts/setup_admin.py       # Grant admin to a Discord ID

# E2E tests (from src/frontend/)
bun x playwright test        # Uses stub_api binary (no real Discord)
```

**VS Code Tasks**: "Run Backend", "Run Frontend", "Generate Test Data", "Build All"

API docs: `http://localhost:5038/docs` (Scalar UI via utoipa)

## Backend Patterns (`src/backend/`)

**Router split**: `main.rs` defines auth + mumble routes and OpenAPI; `lib.rs::get_common_router()` defines shared API routes (me, wallets, roster, notes). The `stub_api` binary reuses `get_common_router()` for E2E testing.

**Auth extractors** (`auth.rs`):
- `AuthenticatedUser` — JWT Bearer validation, extracts `user_id: i64` from claims
- `InternalSecret` — validates `X-Internal-Secret` header for service-to-service calls (Mumble authenticator)

**Admin authorization** — always use `helpers::require_admin_in_tribe()`:
```rust
let (user, tribe, _all_tribes) = require_admin_in_tribe(
    &state.db, auth_user.user_id, params.tribe.as_deref()
).await?;  // Returns ApiResult — (StatusCode, &str) errors
```
This checks: user exists → user is in tribe → user is admin (global `is_admin` OR tribe-level `is_admin`). When tribe is `None` and user admins exactly one tribe, it auto-selects; multiple → 400 error.

**Conventions**:
- `#[utoipa::path(...)]` on all public endpoints; register schemas in `ApiDoc` struct in `main.rs`
- `audit::log_audit()` after all state-changing operations (see `AuditAction` enum)
- User IDs: `i64` serialized as strings via `models::i64_as_string` serde module (prevents JS precision loss)
- Wallet-to-tribe mapping via `FlatLinkedWallet` → grouped into `LinkedWallet { tribes: Vec<String> }` using BTreeMap
- Migrations auto-run on startup via `sqlx::migrate!("./migrations")`
- DB type alias: `db::DbPool` = `Pool<Sqlite>`

**Mumble integration** (`mumble.rs`): Users in the required tribe (env `MUMBLE_REQUIRED_TRIBE`, default "Fire") can create Mumble accounts. Username derived from `wallet_id` in `user_tribes`. Password generated, bcrypt-hashed, stored in `mumble_accounts`. Login verified via `/api/internal/mumble/verify` (protected by `InternalSecret`).

## Frontend Patterns (`src/frontend/`)

- **File-based routing**: TanStack Router — routes in `src/routes/`, auto-generated `routeTree.gen.ts` (do not edit manually)
- **Provider hierarchy**: `ThemeProvider` → `QueryClient` → `SuiClientProvider` → `WalletProvider` → `AuthProvider` → `RouterProvider`
- **Auth**: `useAuth()` hook provides `user`, `token`, `currentTribe`, `login()`, `logout()`, `linkWallet()`, `unlinkWallet()`
- **API base URL**: Currently hardcoded to `http://localhost:5038` in `AuthProvider.tsx`
- **Wallet**: `@mysten/dapp-kit` with `testnet` default network
- **Routes**: `/login`, `/home` (dashboard), `/roster` (admin), `/roster/$id` (member detail), `/voice` (Mumble), `/auth/callback` (OAuth redirect)

## Design System (STRICT)

EVE Frontier-inspired industrial sci-fi aesthetic — reference `docs/design-system.md` and `src/frontend/src/index.css`:

- **ZERO border-radius**: All `--radius-*` vars are `0px !important`
- **Fonts**: `Diskette Mono` (headings/buttons, uppercase), `Inter` (body)
- **Colors**: Use CSS variables (`--bg-primary`, `--text-primary`, etc.) — they auto-switch for light/dark. Brand accent: `--brand-orange: #ff4700`
- **No box shadows on cards** — use 1px borders with `--border-color`
- Theme toggled via `html[data-theme]` attribute (managed by `ThemeProvider`)

## Environment Variables

Backend `.env`:
```
DATABASE_URL=sqlite:void-eid.db
DISCORD_CLIENT_ID=xxx
DISCORD_CLIENT_SECRET=xxx
DISCORD_REDIRECT_URI=http://localhost:5038/api/auth/discord/callback
JWT_SECRET=xxx
FRONTEND_URL=http://localhost:5173
INITIAL_ADMIN_ID=123456789        # Discord ID → auto-grant global admin on login
INTERNAL_SECRET=xxx               # For Mumble authenticator service calls
MUMBLE_REQUIRED_TRIBE=Fire        # Tribe required for Mumble account creation
```

## Testing

- **Backend unit tests**: `#[cfg(test)]` blocks using `sqlite::memory:` — run with `cargo test` from `src/backend/`
- **Frontend unit tests**: Vitest — `bun run test` from `src/frontend/`
- **E2E tests**: Playwright in `src/frontend/e2e/`. Uses `stub_api` binary (`src/backend/src/bin/stub_api.rs`) — seeds an in-memory DB and provides `/api/auth/stub-login?user_id=N` for Discord-free auth

## Key Files

| Concern             | Files                                                      |
| ------------------- | ---------------------------------------------------------- |
| Route registration  | `src/backend/src/main.rs`, `src/backend/src/lib.rs`        |
| Auth + JWT          | `src/backend/src/auth.rs` (extractors: `AuthenticatedUser`, `InternalSecret`) |
| Admin helpers       | `src/backend/src/helpers.rs` (`require_admin_in_tribe`)    |
| Wallet verification | `src/backend/src/wallet.rs` (sui-sdk signature verify)     |
| Mumble accounts     | `src/backend/src/mumble.rs`, `src/murmur/authenticator.py` |
| DB schema           | `src/backend/migrations/01_init.sql` through `04_*`        |
| Frontend auth       | `src/frontend/src/providers/AuthProvider.tsx`               |
| Design tokens       | `src/frontend/src/index.css`, `docs/design-system.md`      |
| E2E stub server     | `src/backend/src/bin/stub_api.rs`                          |
