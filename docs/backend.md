# Backend Documentation

The backend is a Rust application powered by `axum`. It serves as the authoritative source for user authentication and wallet linking.

## Architecture

The backend follows a standard layered architecture:

- **Router** (`main.rs`): Defines API endpoints and middleware (CORS, State).
- **Handlers** (`models/`, `auth/`, `wallet/`): Business logic for specific features.
- **State** (`state.rs`): Application state (Database connection pool).
- **Database** (`db.rs`): Database initialization and interaction helpers using SQLx.
- **OpenAPI** (`main.rs`): API specification generation.

## Dependencies

Key crates used:

- `axum`: Web framework.
- `tokio`: Async runtime.
- `sqlx`: Database interface (SQLite).
- `sui-sdk`: Interaction with Sui blockchain.
- `utoipa`: OpenAPI spec generation from Rust code.
- `reqwest`: HTTP client (for Discord OAuth).
- `jsonwebtoken`: JWT generation and validation.

## API Endpoints

Full API documentation is available via Scalar UI at `/docs` when the server is running.

### Authentication (`/api/auth`)

- `GET /api/auth/discord/login`: Redirects user to Discord OAuth authorization URL.
- `GET /api/auth/discord/callback`: Handles the OAuth callback, creates/updates user, and issues a JWT.
- `GET /api/me`: Returns the currently authenticated user's profile.

### Wallet Management (`/api/wallets`)

- `POST /api/wallets/link-nonce`: Step 1 of linking. Generates a random nonce for the user to sign with their Sui wallet.
- `POST /api/wallets/link-verify`: Step 2 of linking. Verifies the signature of the nonce against the wallet address. If valid, links the wallet to the user.
- `DELETE /api/wallets/:id`: Unlinks a specific wallet.

## Database Schema

The application uses SQLite. Ensure `sqlx-cli` is installed if you need to run migrations manually.

Main Tables:

- `users`: Stores Discord ID and profile info.
- `wallets`: Stores linked Sui addresses, associated with a user ID.

### Database Migrations

The project uses `sqlx` migrations to manage the database schema. Migrations are located in `src/backend/migrations/`.
They are applied automatically on server startup.

To create a new migration (requires `sqlx-cli`):

```bash
sqlx migrate add <name>
```

Then edit the generated `.sql` file.

## Configuration

Environment variables (`.env`):

| Variable                    | Description                                                            | Default/Required          |
| --------------------------- | ---------------------------------------------------------------------- | ------------------------- |
| `DATABASE_URL`              | Connection string for SQLite                                           | `sqlite:void-eid.db`      |
| `JWT_SECRET`                | Secret key for signing JWTs (generate via `openssl rand -base64 32`)  | **Required**              |
| `DISCORD_CLIENT_ID`         | OAuth2 Client ID from Discord                                          | **Required**              |
| `DISCORD_CLIENT_SECRET`     | OAuth2 Client Secret                                                   | **Required**              |
| `DISCORD_REDIRECT_URI`      | Oauth2 Redirect URI (e.g., `http://localhost:5038/api/auth/callback`)  | **Required**              |
| `FRONTEND_URL`              | URL of the frontend (for CORS and redirects)                           | `http://localhost:5173`   |
| `PORT`                      | Port to listen on                                                      | `5038`                    |
| `INITIAL_ADMIN_ID`          | Discord ID of the initial admin user                                   | _Optional_                |
| `SUPER_ADMIN_DISCORD_IDS`   | Comma-separated list of Super Admin Discord IDs                        | _Optional_                |
| `SUPER_ADMIN_AUDIT_WEBHOOK` | Discord Webhook URL for critical audit alerts                          | _Optional_                |
| `IDENTITY_HASH_PEPPER`      | Secret pepper for hashing denylisted identifiers                       | **Required**              |
| `MUMBLE_REQUIRED_TRIBE`     | The tribe name required to create a Mumble account                     | `Fire`                    |
| `INTERNAL_SECRET`           | Shared secret for Backend-to-Murmur Authenticator communication        | **Required** ⚠️           |
| `ICE_SECRET_READ`           | ICE read secret for Murmur server (required if running Mumble)         | **Required for Mumble**   |
| `ICE_SECRET_WRITE`          | ICE write secret for Murmur server (required if running Mumble)        | **Required for Mumble**   |
| `ICE_SECRET`                | Legacy ICE secret reference (set to same value as `ICE_SECRET_WRITE`)  | **Required for Mumble**   |

⚠️ **Security Notice**: As of the 2026-02-14 security audit remediation, `INTERNAL_SECRET`, `ICE_SECRET_READ`, and `ICE_SECRET_WRITE` **must** be set to strong random values. The application will fail to start if `INTERNAL_SECRET` is missing. Generate secrets using:
```bash
openssl rand -base64 32
```

## Authentication Flow

1. **Discord Login**: User clicks "Login with Discord". Frontend opens `/api/auth/discord/login`.
2. **Redirect**: Backend redirects to Discord.
3. **Callback**: Discord redirects back to `/api/auth/discord/callback` with a code.
4. **Token Exchange**: Backend exchanges code for access token, fetches user info from Discord.
5. **Session**: Backend mints a JWT and returns it (often as a cookie or query param to frontend).

## Wallet Linking Flow

1. **Request Nonce**: Frontend requests a challenge user `POST /api/wallets/link-nonce`.
2. **Sign**: User signs the message (nonce) using their Sui Wallet (via dApp Kit).
3. **Verify**: Frontend sends signature and public key to `POST /api/wallets/link-verify`.
4. **Link**: Backend verifies signature using `sui-sdk` or crypto libs. If valid, the address is saved to the DB.
