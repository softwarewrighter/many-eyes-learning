#!/bin/bash
# Run the Many-Eyes Learning backend server

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Activate venv if it exists
if [ -d ".venv" ]; then
    source .venv/bin/activate
fi

# Run the server
echo "Starting Many-Eyes Learning API server on http://localhost:8000"
python -m uvicorn web.api.main:app --reload --host 0.0.0.0 --port 8000
