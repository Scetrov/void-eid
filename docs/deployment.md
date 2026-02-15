# Deployment Guide

This guide covers how to build and deploy the Void eID application.

## Strategy

The application consists of two parts:

1.  **Rust Backend**: A compiled binary.
2.  **React Frontend**: Static HTML/CSS/JS assets.

**Recommended Deployment**:
Run the Rust binary as a service (e.g., systemd, Docker). Serve the frontend static assets either via a reverse proxy (Nginx) or by embedding them into the Rust backend (if configured to serve static files).

## Building the Backend

1.  **Environment**: The CI pipeline builds a glibc binary on Ubuntu. Production Docker images use `debian:trixie-slim` as the runtime base.
2.  **Build**:
    ```bash
    cd src/backend
    cargo build --release
    ```
3.  **Artifact**: The binary will be at `target/release/void-eid-backend`.

## Building the Frontend

1.  **Build**:
    ```bash
    cd src/frontend
    bun install
    bun run build
    ```
2.  **Artifact**: The static files will be in `src/sui/dist`.

## Deployment with Docker (Recommended)

The backend Docker image uses a pre-built binary on `debian:trixie-slim`. The CI pipeline builds the binary and packages it — no Rust compilation happens inside Docker.

```dockerfile
# Backend runtime image (src/backend/Dockerfile)
FROM debian:trixie-slim
WORKDIR /usr/local/bin
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 libsqlite3-0 && \
    rm -rf /var/lib/apt/lists/*
COPY void-eid-backend .
RUN chmod +x ./void-eid-backend
EXPOSE 5038
CMD ["./void-eid-backend"]
```

## Manual Deployment (Linux/Systemd)

1.  **Database**:
    Ensure the directory for the SQLite database exists and is writable by the service user.
    Run migrations if necessary (or rely on app startup if it handles it).

2.  **Frontend serving via Nginx**:

    ```nginx
    server {
        listen 80;
        server_name your-domain.com;

        location / {
            root /var/www/void-eid/dist;
            try_files $uri $uri/ /index.html;
        }

        location /api/ {
            proxy_pass http://localhost:5038;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
    ```

## Environment Configuration

### Required Variables

The following environment variables **must** be set in production:

- `JWT_SECRET`: Strong random secret for JWT signing (generate via `openssl rand -base64 32`)
- `IDENTITY_HASH_PEPPER`: Strong random secret for identity hashing (generate via `openssl rand -base64 32`)
- `INTERNAL_SECRET`: Shared secret for backend-to-Murmur authenticator communication (generate via `openssl rand -base64 32`)
- `DISCORD_CLIENT_ID`: Your Discord OAuth2 Application Client ID
- `DISCORD_CLIENT_SECRET`: Your Discord OAuth2 Application Client Secret
- `DISCORD_REDIRECT_URI`: Must match the production URL registered in Discord Developer Portal (e.g., `https://yourdomain.com/api/auth/discord/callback`)

### Mumble/Murmur Variables (if running voice server)

If deploying with Mumble voice integration, also set:

- `ICE_SECRET_READ`: ICE read secret for Murmur server (generate via `openssl rand -base64 32`)
- `ICE_SECRET_WRITE`: ICE write secret for Murmur server (generate via `openssl rand -base64 32`)
- `MUMBLE_REQUIRED_TRIBE`: Tribe name required for Mumble access (default: `Fire`)

### Optional Variables

- `DATABASE_URL`: SQLite connection string (default: `sqlite:void-eid.db?mode=rwc`)
- `PORT`: Backend listening port (default: `5038`)
- `FRONTEND_URL`: Frontend URL for CORS (default: `http://localhost:5173`)
- `INITIAL_ADMIN_ID`: Discord User ID to grant initial admin access
- `SUPER_ADMIN_DISCORD_IDS`: Comma-separated Discord User IDs for super admin access
- `SUPER_ADMIN_AUDIT_WEBHOOK`: Discord webhook URL for critical audit alerts

### Security Best Practices

1. **Never commit secrets to version control** - Use `.env` files (gitignored) or secure secret management systems
2. **Generate strong random secrets** - Always use `openssl rand -base64 32` or equivalent
3. **Rotate secrets regularly** - Especially after personnel changes or suspected compromise
4. **Fail fast** - The application is configured to panic on startup if required secrets are missing
5. **Use different secrets per environment** - Dev, staging, and production should have unique secrets
