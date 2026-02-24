#!/bin/bash
# Run the Many-Eyes Learning frontend development server

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/frontend"

echo "Starting Many-Eyes Learning frontend on http://localhost:8080"
trunk serve
