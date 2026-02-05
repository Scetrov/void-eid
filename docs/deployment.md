# Deployment Guide

This guide covers how to build and deploy the Void eID application.

## Strategy

The application consists of two parts:

1.  **Rust Backend**: A compiled binary.
2.  **React Frontend**: Static HTML/CSS/JS assets.

**Recommended Deployment**:
Run the Rust binary as a service (e.g., systemd, Docker). Serve the frontend static assets either via a reverse proxy (Nginx) or by embedding them into the Rust backend (if configured to serve static files).

## Building the Backend

1.  **Environment**: Ensure target environment has `libc` compatible with the build (or build static musl binary).
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

A multi-stage Dockerfile is the easiest way to deploy.

```dockerfile
# Build Backend
FROM rust:latest as backend-builder
WORKDIR /app
COPY src/backend .
RUN cargo build --release

# Build Frontend
FROM node:20 as frontend-builder
WORKDIR /app
COPY src/frontend .
RUN bun install && bun run build

# Runtime
FROM debian:bookworm-slim
WORKDIR /app
# Install dependencies (openssl, ca-certificates, sqlite3)
RUN apt-get update && apt-get install -y libssl3 ca-certificates sqlite3 && rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder /app/target/release/void-eid-backend .
# Optional: If backend serves frontend
# COPY --from=frontend-builder /app/dist ./static

ENV PORT=8080
ENV DATABASE_URL=sqlite:/data/void-eid.db

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

Ensure production `.env` variables are set:

- `JWT_SECRET`: specific long random string.
- `DISCORD_REDIRECT_URI`: Must match the production URL registered in Discord Developer Portal.
