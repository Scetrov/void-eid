# Frontend Documentation

The frontend is a Single Page Application (SPA) built with React 19 and Vite. It serves as the user interface for claiming roles and managing wallets.

## Tech Stack

- **Framework**: [Vite](https://vitejs.dev/) + [React](https://react.dev/) (TypeScript)
- **Routing**: [@tanstack/react-router](https://tanstack.com/router)
- **State/Data Fetching**: [@tanstack/react-query](https://tanstack.com/query)
- **Web3**: [@mysten/dapp-kit](https://sdk.mystenlabs.com/dapp-kit) for Sui Wallet connection.
- **Styling**: Vanilla CSS with strict Design System variables (see `design-system.md`) + [Lucide React](https://lucide.dev/) for icons.
- **Validation**: [Zod](https://zod.dev/).

## Project Structure (`src/frontend`)

```text
src/
├── components/     # Reusable UI components (Buttons, Cards, Modals)
├── hooks/          # Custom React hooks
├── routes/         # Route definitions (part of TanStack router)
├── lib/            # Utilities and helper functions
├── main.tsx        # Entry point
├── App.tsx         # Main App component & Provider setup
└── index.css       # Global styles and CSS variables
```

## Key Features

### Wallet Connection

Uses `dapp-kit`'s `ConnectButton` and `useWallet` hook to manage wallet state.

- **Provider**: Wrapped in `SuiClientProvider` and `WalletProvider`.
- **Auto-connect**: Configured to attempt auto-connection on load.

### Authentication

Authentication state is managed via React Query.

- **Login**: Redirects to backend Discord endpoint.
- **Session**: Checks `/api/me` to validate session and get user details.

### Routing

We use file-based routing features or code-based routing from TanStack Router.

- `routes/` directory contains route definitions.
- `__root.tsx` usually defines the layout shell.

## Styles

The application uses a strict set of CSS variables defined in `index.css`.

- **Design Philosophy**: High-contrast, technical, "sci-fi industrial" aesthetic.
- **Rules**:
  - No border-radius (0px).
  - Monospace fonts for headers/data.
  - Specific "stone" and "nebula" color palettes.

See [Design System](./design-system.md) for variable references.

## Development

### Commands

```bash
# Start dev server
bun run dev

# Build for production
bun run build

# Preview production build
bun run preview

# Lint code
bun run lint
```

### Environment Variables

Create a `.env` file in `src/sui` if needed (Vite requires `VITE_` prefix for client-exposed variables).

| Variable       | Description                              |
| -------------- | ---------------------------------------- |
| `VITE_API_URL` | URL of the backend API (if not proxying) |

## Troubleshooting

### Auth Loop: Logged In Then Bounced to Login

**Symptom**: After Discord OAuth callback, you see a 400 error for `/api/auth/exchange` and get redirected back to login.

**Cause**: React 19 Strict Mode double-invokes effects in development. The auth code is consumed on first call, second call fails.

**Fix**: The callback route (`src/routes/auth/callback.tsx`) uses a `useRef` to prevent duplicate exchange attempts. If you see this, ensure the ref pattern is implemented:

```tsx
const hasExchangedRef = useRef(false);
// ... in useEffect:
if (hasExchangedRef.current) return;
hasExchangedRef.current = true;
```

**Note**: Auth codes have a **2-minute TTL** from creation. If you see `"Code expired"` errors, the code took longer than 2 minutes to exchange (unlikely in normal flow).
