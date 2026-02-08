#!/bin/bash
set -e

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
export ICE_SECRET="secret"
python3 /app/authenticator.py &
AUTH_PID=$!

# Wait for both
wait $MURMUR_PID $AUTH_PID
