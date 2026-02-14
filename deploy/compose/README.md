# Void eID Docker Compose Deployment

## Quick Start

This deployment uses the workspace root `.env` file (located at `/home/scetrov/source/void-eid/.env`).

1. **Ensure `.env` is configured** in the workspace root (two levels up from this directory)
   - The `.env` file should already exist with your configuration
   - If not, copy from `.env.example` in the workspace root

2. **Start the services:**
   ```bash
   docker compose up -d
   ```

## Environment Configuration

This compose file references `../../.env` (the workspace root `.env` file).

### Required Discord OAuth Configuration

Register an application at https://discord.com/developers/applications

- **`DISCORD_CLIENT_ID`**: Your Discord Application's Client ID
  - Found in: OAuth2 → General → Client ID

- **`DISCORD_CLIENT_SECRET`**: Your Discord Application's Client Secret
  - Found in: OAuth2 → General → Client Secret
  - Click "Reset Secret" if needed

- **OAuth2 Redirect URL**: Add this to your Discord app:
  - `http://localhost:5038/api/auth/discord/callback`
  - Found in: OAuth2 → Redirects

### Required Admin Configuration

- **`INITIAL_ADMIN_ID`**: Your Discord User ID
  - Enable Developer Mode in Discord (Settings → Advanced → Developer Mode)
  - Right-click your username → Copy User ID

### Required Secrets (Generate Random Values)

Generate secure random strings for these values using `openssl rand -base64 32`:

- **`JWT_SECRET`**: Secret for signing JWT tokens
  ```bash
  openssl rand -base64 32
  ```

- **`IDENTITY_HASH_PEPPER`**: Secret pepper for hashing denylisted identifiers
  ```bash
  openssl rand -base64 32
  ```

- **`INTERNAL_SECRET`**: **REQUIRED** - Shared secret for backend-to-Murmur authenticator API calls
  ```bash
  openssl rand -base64 32
  ```
  ⚠️ The application will **fail to start** if this is not set.

- **`ICE_SECRET_READ`**: **REQUIRED for Mumble** - ICE read secret for Murmur server
  ```bash
  openssl rand -base64 32
  ```

- **`ICE_SECRET_WRITE`**: **REQUIRED for Mumble** - ICE write secret for Murmur server
  ```bash
  openssl rand -base64 32
  ```

- **`ICE_SECRET`**: Legacy reference - set to same value as `ICE_SECRET_READ`
  ```bash
  # Use the same value as ICE_SECRET_READ
  ```

### Optional Configuration

- **`SUPER_ADMIN_DISCORD_IDS`**: Comma-separated Discord IDs for super admins
  - Example: `123456789,987654321`
  - Leave empty if only using `INITIAL_ADMIN_ID`

- **`SUPER_ADMIN_AUDIT_WEBHOOK`**: Discord webhook URL for super admin audit logs
  - Leave empty to disable

- **`MUMBLE_REQUIRED_TRIBE`**: Tribe required for Mumble account creation
  - Default: `Fire`

## Pre-configured Values (No Changes Needed)

The following are already set correctly in `.env.sample`:

- `DISCORD_REDIRECT_URI=http://localhost:5038/api/auth/discord/callback`
- `FRONTEND_URL=http://localhost:5173`
- `DATABASE_URL=sqlite:///data/void-eid.db?mode=rwc`
- `BACKEND_URL=http://backend:5038/api/internal/mumble`
- `VITE_API_URL=http://localhost:5038`
- `PORT=5038`

## Starting the Services

```bash
docker compose up -d
```

## Checking Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f backend
docker compose logs -f frontend
docker compose logs -f murmur
```

## Stopping the Services

```bash
docker compose down
```

## Security Notes

- **Never commit `.env` to version control** (it's in `.gitignore`)
- Generate unique secrets for each deployment
- Use strong random values for all secret keys
- Restrict `SUPER_ADMIN_DISCORD_IDS` to trusted users only
