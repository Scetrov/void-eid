# Void eID Docker Compose Deployment

## Quick Start

1. **Copy the environment template:**
   ```bash
   cp .env.sample .env
   ```

2. **Configure required values in `.env`:**

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

Generate secure random strings for these values:

- **`JWT_SECRET`**: Secret for signing JWT tokens (min 32 characters)
  ```bash
  openssl rand -base64 32
  ```

- **`ICE_SECRET`**: Secret for Murmur ICE protocol (min 32 characters)
  ```bash
  openssl rand -base64 32
  ```

- **`INTERNAL_SECRET`**: Secret for internal service communication (min 32 characters)
  ```bash
  openssl rand -base64 32
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
