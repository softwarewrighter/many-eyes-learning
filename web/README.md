# Many-Eyes Learning Web Visualization

Real-time visualization of multi-scout reinforcement learning training.

## Quick Start

```bash
# From project root
uv venv
source .venv/bin/activate
uv pip install -e ".[web]"

# Install Rust/Trunk (if not already installed)
cargo install trunk wasm-bindgen-cli

# Build and serve on http://localhost:3200
./scripts/serve.sh
```

## Architecture

```
┌─────────────────────────┐
│   Browser               │
│   http://localhost:3200 │
├─────────────────────────┤
│   FastAPI Server        │
│   - Static files (/)    │
│   - REST API (/api/*)   │
│   - WebSocket (/ws/*)   │
└─────────────────────────┘
```

The server runs on a single port (3200) and serves:
- Frontend static files (built Yew/WASM app)
- REST API for experiments
- WebSocket for real-time training events

## Features

- **Real-time Training**: Watch scouts explore the grid world live
- **Multiple Scouts**: Visualize 1-5 scouts with different exploration strategies
- **Training Controls**: Start/pause/stop with adjustable speed
- **Metrics Panel**: Live success rate, loss, and episode tracking
- **Learning Curves**: Canvas-based chart of training progress
- **Scout Legend**: Color-coded scouts with individual statistics

## Development

### Build Frontend Only

```bash
cd web/frontend
trunk build          # Dev build
trunk build --release  # Production build
```

### Run Server Only

```bash
source .venv/bin/activate
python -m uvicorn web.api.main:app --port 3200
```

### API Endpoints

**REST:**
- `GET /api/experiments` - List saved experiments
- `GET /api/experiments/{id}` - Get experiment data for replay
- `GET /api/grid-info` - Get grid configuration and scout colors
- `GET /api/health` - Health check

**WebSocket:**

Connect to `/ws/train/{client_id}` and send commands:

```json
{"command": "start", "config": {"n_scouts": 5, "grid_size": 5, "episodes": 100}}
{"command": "pause"}
{"command": "resume"}
{"command": "set_speed", "speed": 2.0}
{"command": "stop"}
```

## File Structure

```
web/
├── api/                    # Python FastAPI backend
│   ├── main.py             # App entry point (serves on port 3200)
│   ├── routes/
│   │   ├── experiments.py  # REST: list/load experiments
│   │   └── training.py     # WebSocket: real-time training
│   ├── models/
│   │   └── events.py       # Pydantic event models
│   └── services/
│       └── streaming_trainer.py
│
├── frontend/               # Yew/WASM frontend
│   ├── Cargo.toml
│   ├── src/
│   │   ├── app.rs          # Root component
│   │   ├── components/     # UI components
│   │   ├── services/       # WebSocket client
│   │   └── types/          # Event structs
│   └── static/
│       └── styles.css
│
└── requirements.txt        # Python dependencies
```
