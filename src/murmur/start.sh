#!/bin/bash
set -e

# Start Murmur in background
echo "Starting Murmur..."
murmurd -ini /etc/murmur.ini -fg &
MURMUR_PID=$!

# Wait for Ice to be ready
echo "Waiting for Ice..."
sleep 5

# Start Authenticator
echo "Starting Authenticator..."
export ICE_SECRET="secret"
python3 /app/authenticator.py &
AUTH_PID=$!

# Wait for both
wait $MURMUR_PID $AUTH_PID
