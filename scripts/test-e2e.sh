#!/bin/bash
set -e

# Get absolute path to project root
SCRIPT_DIR=$(dirname "$(realpath "$0")")
PROJECT_ROOT="$SCRIPT_DIR/.."

echo "Project Root: $PROJECT_ROOT"

# Build Stub API once (Playwright will start it via webServer config)
echo "Building Stub API..."
cd "$PROJECT_ROOT/src/backend"
cargo build --bin stub_api

# Run Tests (Playwright will start both frontend and stub API)
echo "Running Playwright Tests..."
cd "$PROJECT_ROOT/src/frontend"
bun x playwright test "$@"

echo "Tests completed successfully."
