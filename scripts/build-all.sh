#!/bin/bash
set -e

# Check for bun
if ! command -v bun &> /dev/null; then
    echo "Error: 'bun' command not found. Please install bun."
    exit 1
fi

echo "Building Frontend with Bun..."
cd src/frontend
bun install
bun run build
cd ../..

echo "Building Backend..."
cd src/backend
cargo build --release
cd ../..

echo "Build Complete!"
