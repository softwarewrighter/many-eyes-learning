#!/bin/bash
# Serve Many-Eyes Learning web visualization on port 3200
#
# This script builds and runs the Rust backend which serves the frontend.
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

echo -e "${BLUE}Building backend...${NC}"
cd web/backend
cargo build --release
cd "$PROJECT_ROOT"

echo -e "${GREEN}Starting server on http://localhost:3200${NC}"
./web/backend/target/release/many-eyes-server
