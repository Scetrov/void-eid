# Void eID Quadlet Deployment

This directory contains Podman Quadlet files for deploying the Void eID stack using `systemd`. Unlike standard Docker Compose, Quadlet allows the container lifecycle to be managed natively by the host's init system.

## Setup Instructions

### 1. Prerequisites

- **Podman** installed on the host.
- A user-level `systemd` instance.
- The `.env` file must be located at `~/.config/void-eid/.env`.

### 2. Prepare Environment

Ensure the configuration directory exists and contains your environment variables:

```bash
mkdir -p ~/.config/void-eid
cp .env ~/.config/void-eid/.env
```

### 3. Deploy Quadlet Files

Copy the `.container` and `.network` files to your user's systemd directory:

```bash
mkdir -p ~/.config/containers/systemd
cp deploy/quadlet/* ~/.config/containers/systemd/
```

### 4. Enable and Start Services

Reload the systemd daemon to pick up the new Quadlet files and start the services:

```bash
systemctl --user daemon-reload
systemctl --user enable --now void-backend.service void-murmur.service void-frontend.service
```

## Architecture

- **Network**: All containers run on the `void.network` (defined in `void.network`), providing internal DNS.
- **Backend API**: Accessible at port `5038`.
- **Frontend Dashboard**: Accessible at port `5173`.
- **Murmur Service**: VOIP server accessible at port `64738` (TCP/UDP).

## Troubleshooting

Check the logs of the systemd services:

```bash
journalctl --user -u void-backend.service -f
```

Or check container status directly:

```bash
podman ps
```

## Note on External Access

If you are accessing the dashboard from a different machine, ensure the `VITE_API_URL` environment variable in `void-frontend.container` points to the external IP address of this server.
