#!/bin/bash
set -e

# Cleanup function
cleanup() {
    echo "Stopping processes..."
    # Kill background jobs
    kill $(jobs -p) 2>/dev/null || true
}
trap cleanup EXIT

# Get absolute path to project root
SCRIPT_DIR=$(dirname "$(realpath "$0")")
PROJECT_ROOT="$SCRIPT_DIR/.."

echo "Project Root: $PROJECT_ROOT"

# 1. Build Stub API
echo "Building Stub API..."
cd "$PROJECT_ROOT/src/backend"
cargo build --bin stub_api

# 2. Start Stub API
echo "Starting Stub API on port 5039..."
PORT=5039 \
DATABASE_URL="sqlite::memory:" \
JWT_SECRET="test-secret" \
FRONTEND_URL="http://localhost:5178" \
./target/debug/stub_api &

# Wait helper
wait_for_port() {
  local port=$1
  local retries=30
  echo "Waiting for port $port..."
  while ! nc -z localhost $port; do
    sleep 0.5
    retries=$((retries - 1))
    if [ $retries -eq 0 ]; then
      echo "Timed out waiting for port $port"
      exit 1
    fi
  done
  echo "Port $port is ready."
}

wait_for_port 5039

# 3. Start Frontend
echo "Starting Frontend..."
cd "$PROJECT_ROOT/src/frontend"
# Use port 5178
./node_modules/.bin/vite --mode test --port 5178 &

wait_for_port 5178

# 4. Run Tests
echo "Running Playwright Tests..."
export BASE_URL="http://localhost:5178"
npx playwright test "$@"

echo "Tests completed successfully."
