#!/bin/bash
set -e

# Validate required secrets are set
if [ -z "$ICE_SECRET_READ" ]; then
    echo "ERROR: ICE_SECRET_READ environment variable is not set"
    exit 1
fi

if [ -z "$ICE_SECRET_WRITE" ]; then
    echo "ERROR: ICE_SECRET_WRITE environment variable is not set"
    exit 1
fi

# Template murmur.ini with ICE secrets
echo "Configuring Murmur with ICE secrets from environment..."
cat >> /etc/murmur.ini << EOF
icesecretread=${ICE_SECRET_READ}
icesecretwrite=${ICE_SECRET_WRITE}
EOF

# Fix permissions for data dir
chown -R mumble-server:mumble-server /data

# Start Murmur in background
echo "Starting Murmur..."
mumble-server -ini /etc/murmur.ini -fg &
MURMUR_PID=$!

# Wait for Ice to be ready
echo "Waiting for Ice..."
timeout 30s bash -c 'until python3 -c "import socket; s = socket.create_connection((\"127.0.0.1\", 6502), timeout=1); s.close()" 2>/dev/null; do echo "Waiting for port 6502..."; sleep 1; done'
echo "Ice is ready."

# Start Authenticator
echo "Starting Authenticator..."
# ICE_SECRET env var is used by authenticator.py (needs WRITE secret to modify server)
export ICE_SECRET="${ICE_SECRET_WRITE}"
python3 /app/authenticator.py &
AUTH_PID=$!

# Wait for both
wait $MURMUR_PID $AUTH_PID
