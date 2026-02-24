#!/bin/bash
# Serve Many-Eyes Learning web visualization on port 3200
#
# This script builds the frontend and serves everything from a single port.
# Usage: ./scripts/serve.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Building frontend...${NC}"
cd web/frontend
trunk build --release
cd "$PROJECT_ROOT"

echo -e "${BLUE}Starting server on http://localhost:3200${NC}"
echo -e "${GREEN}Open http://localhost:3200 in your browser${NC}"

# Activate venv if it exists
if [ -d ".venv" ]; then
    source .venv/bin/activate
fi

# Run the server
python -m uvicorn web.api.main:app --host 0.0.0.0 --port 3200
