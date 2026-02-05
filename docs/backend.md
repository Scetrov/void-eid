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

| Variable                | Description                           | Default              |
| ----------------------- | ------------------------------------- | -------------------- |
| `DATABASE_URL`          | Connection string for SQLite          | `sqlite:void-eid.db` |
| `jwt_secret`            | Secret key for signing JWTs           | _Required_           |
| `DISCORD_CLIENT_ID`     | OAuth2 Client ID from Discord         | _Required_           |
| `DISCORD_CLIENT_SECRET` | OAuth2 Client Secret                  | _Required_           |
| `PORT`                  | Port to listen on                     | `5038`               |
| `BASE_URL`              | Public URL of the API (for redirects) | _Optional_           |
| `INITIAL_ADMIN_ID`      | Discord ID of the initial admin user  | _Optional_           |

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
