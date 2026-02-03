# Void eID - Copilot Instructions

## Architecture Overview

This is a **Discord-to-Sui Wallet identity verification system** with:

- **Backend** (`src/rust/`): Rust/Axum API with SQLite, handling Discord OAuth2 and Sui wallet signature verification
- **Frontend** (`src/sui/`): React 19/Vite SPA using TanStack Router + Sui dApp Kit for wallet connections

**Core Flow**: User authenticates via Discord → links Sui wallet by signing a nonce → verified wallets stored in DB

## Development Commands

```bash
# Backend (from src/rust/)
cargo run                    # Start API at localhost:5038
cargo fmt && cargo clippy    # Required before commits

# Frontend (from src/sui/)
npm run dev                  # Start dev server at localhost:5173
npm run lint                 # ESLint check

# Full stack (from root)
docker compose up            # Run both services with hot-reload
```

API docs available at `http://localhost:5038/docs` (Scalar UI via Utoipa)

## Backend Patterns (`src/rust/`)

- **Layered architecture**: Routes in `main.rs`, handlers in feature modules (`auth.rs`, `wallet.rs`, `roster.rs`)
- **State**: `AppState` holds DB pool + in-memory nonce map for wallet linking (`state.rs`)
- **Auth extraction**: Use `AuthenticatedUser` extractor for protected routes (see `auth.rs:AuthenticatedUser`)
- **OpenAPI**: Add `#[utoipa::path(...)]` annotations to all public endpoints, register schemas in `ApiDoc` struct
- **Migrations**: Auto-run on startup via `sqlx::migrate!("./migrations")`. Add new ones with `sqlx migrate add <name>`

```rust
// Protected endpoint pattern
pub async fn my_handler(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,  // JWT validation + user extraction
) -> impl IntoResponse { ... }
```

## Frontend Patterns (`src/sui/`)

- **File-based routing**: TanStack Router with routes in `src/routes/`. Route tree auto-generated to `routeTree.gen.ts`
- **Provider hierarchy**: `ThemeProvider` → `AppProviders` (QueryClient + SuiClientProvider + WalletProvider + AuthProvider)
- **Auth state**: Via `AuthProvider` context - use `useAuth()` hook for `user`, `token`, `login()`, `logout()`, `linkWallet()`
- **Wallet integration**: `@mysten/dapp-kit` handles wallet connections. Default network: `testnet`

## Strict Design System

The UI follows an **EVE Frontier-inspired industrial sci-fi aesthetic**:

- **ZERO border-radius**: All elements use `0px` radius - enforced via `!important` in CSS
- **Fonts**: `Diskette Mono` for headings/buttons, `Inter` for body text
- **Colors**: Stone palette (light) / deep red-black nebula (dark), `--brand-orange: #ff7400` for accents
- **Buttons**: Uppercase, monospace, angular with 1px borders

Reference [design-system.md](docs/design-system.md) and [index.css](src/sui/src/index.css) for exact values.

## Environment Variables

Backend requires in `.env`:

```
DATABASE_URL=sqlite:void-eid.db
DISCORD_CLIENT_ID=xxx
DISCORD_CLIENT_SECRET=xxx
DISCORD_REDIRECT_URI=http://localhost:5038/api/auth/discord/callback
JWT_SECRET=xxx
```

## Testing

- **E2E tests**: Playwright in `src/sui/e2e/`. Run with `npx playwright test`
- **Backend tests**: In-module `#[cfg(test)]` blocks using SQLite `:memory:` databases

## Key Files to Reference

| Concern           | Files                                                                                             |
| ----------------- | ------------------------------------------------------------------------------------------------- |
| API routes        | [main.rs](../src/rust/src/main.rs)                                                                |
| Auth flow         | [auth.rs](../src/rust/src/auth.rs), [AuthProvider.tsx](../src/sui/src/providers/AuthProvider.tsx) |
| Wallet linking    | [wallet.rs](../src/rust/src/wallet.rs) - nonce/signature verification using `sui-sdk`             |
| Database schema   | [migrations/](../src/rust/migrations/)                                                            |
| CSS variables     | [index.css](../src/sui/src/index.css)                                                             |
| Route definitions | [src/routes/](../src/sui/src/routes/)                                                             |
