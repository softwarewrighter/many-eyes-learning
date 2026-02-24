#!/bin/bash
# Development mode: run frontend and backend with hot reload
#
# Usage: ./scripts/dev.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "Starting development servers..."
echo "Frontend: http://localhost:8080 (trunk serve)"
echo "Backend:  http://localhost:3200 (cargo run)"
echo ""
echo "Press Ctrl+C to stop"

# Run backend in background
cd web/backend
cargo run &
BACKEND_PID=$!

# Run frontend (trunk serve)
cd ../frontend
trunk serve --port 8080 &
FRONTEND_PID=$!

# Wait and cleanup
trap "kill $BACKEND_PID $FRONTEND_PID 2>/dev/null" EXIT
wait
