# Many-Eyes Learning Web Visualization

Real-time visualization of multi-scout reinforcement learning training.

## Quick Start

### 1. Install Dependencies

```bash
# From project root
uv venv
source .venv/bin/activate
uv pip install -e ".[web]"

# Install Rust/Trunk (if not already installed)
cargo install trunk wasm-bindgen-cli
```

### 2. Run Both Servers

**Terminal 1 - Backend:**
```bash
./web/run_backend.sh
# Or manually:
python -m uvicorn web.api.main:app --reload --port 8000
```

**Terminal 2 - Frontend:**
```bash
./web/run_frontend.sh
# Or manually:
cd web/frontend && trunk serve
```

### 3. Open Browser

Navigate to http://localhost:3200

## Architecture

```
┌─────────────────────────┐       ┌─────────────────────────┐
│   Yew/WASM Frontend     │◄─────►│   FastAPI Backend       │
│   (localhost:3200)      │  WS   │   (localhost:8000)      │
│                         │       │                         │
│ - Grid visualization    │       │ - /ws/train (realtime)  │
│ - Training controls     │       │ - /api/experiments      │
│ - Metrics panel         │ REST  │ - Training runner       │
│ - Learning curves       │◄─────►│                         │
└─────────────────────────┘       └─────────────────────────┘
```

## Features

- **Real-time Training**: Watch scouts explore the grid world live
- **Multiple Scouts**: Visualize 1-5 scouts with different exploration strategies
- **Training Controls**: Start/pause/stop with adjustable speed
- **Metrics Panel**: Live success rate, loss, and episode tracking
- **Learning Curves**: Canvas-based chart of training progress
- **Scout Legend**: Color-coded scouts with individual statistics

## API Endpoints

### REST

- `GET /api/experiments` - List saved experiments
- `GET /api/experiments/{id}` - Get experiment data for replay
- `GET /api/grid-info` - Get grid configuration and scout colors

### WebSocket

Connect to `/ws/train/{client_id}` and send commands:

```json
// Start training
{"command": "start", "config": {"n_scouts": 5, "grid_size": 5, "episodes": 100}}

// Control training
{"command": "pause"}
{"command": "resume"}
{"command": "set_speed", "speed": 2.0}
{"command": "stop"}
```

## Development

### Backend (Python)

```bash
cd web/api
python -c "from main import app; print(app.routes)"
```

### Frontend (Rust/Yew)

```bash
cd web/frontend
cargo check        # Type check
trunk build        # Build WASM
trunk serve        # Dev server with hot reload
```

## File Structure

```
web/
├── api/                    # Python FastAPI backend
│   ├── main.py             # App entry point
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
├── run_backend.sh          # Backend start script
├── run_frontend.sh         # Frontend start script
└── requirements.txt        # Python dependencies
```
