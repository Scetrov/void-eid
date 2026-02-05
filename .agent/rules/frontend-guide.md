---
trigger: glob
globs: src/frontend/**
---

## Preferences

- Always use `bun` rather than `pnpm` or `npm`.
- Always use clear variable and method names rather than using inline comments.
- Always write Unit Tests for new code.
- Only add Integration Tests when testing integration, never for something that can be unit tested
- Always write Playwright tests for user jounies.
- Always use the Stub API for Playwright tests.

## Patterns

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
