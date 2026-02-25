# Many-Eyes Learning Web Visualization

Real-time visualization of multi-scout reinforcement learning training.

**Stack:** Rust + Yew/WASM (frontend) + Axum (backend)

## Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install trunk (Yew build tool)
cargo install trunk wasm-bindgen-cli
```

## Quick Start

```bash
# From repo root - builds frontend and starts server on http://localhost:3200
./scripts/serve.sh
```

## Manual Build & Run

```bash
# 1. Build frontend (WASM)
cd web/frontend
trunk build --release
# Output: web/frontend/dist/

# 2. Build and run backend
cd web/backend
cargo build --release
./target/release/many-eyes-server
# Server runs on http://localhost:3200, serves frontend from ../frontend/dist/
```

## Development Mode

```bash
# Hot reload for frontend development
./scripts/dev.sh

# Or manually:
# Terminal 1 - Backend
cd web/backend && cargo run

# Terminal 2 - Frontend with hot reload
cd web/frontend && trunk serve --proxy-backend=http://localhost:3200/ws
# Frontend: http://localhost:8080 (proxies WebSocket to backend)
```

## Architecture

```
┌─────────────────────────────────────────┐
│   http://localhost:3200                 │
├─────────────────────────────────────────┤
│   Axum Server (Rust)                    │
│                                         │
│   Static Files (Yew/WASM)               │
│   - Grid visualization                  │
│   - Training controls                   │
│   - Metrics panel                       │
│   - Learning curves                     │
│                                         │
│   WebSocket API                         │
│   - /ws/train/{client_id}               │
│   - Real-time training events           │
└─────────────────────────────────────────┘
```

## Features

- **Real-time Training**: Watch scouts explore the grid world live
- **Multiple Scouts**: Visualize 1-5 scouts with different exploration strategies
- **Training Controls**: Start/pause/stop with adjustable speed
- **Metrics Panel**: Live success rate and episode tracking
- **Learning Curves**: Canvas-based chart of training progress (high-DPI aware)
- **Policy Visualization**: Arrow overlay showing learned policy
- **Replay Mode**: Step through recorded training at adjustable speed (0.1x - 10x)
  - Play/Pause/Step controls
  - Timeline scrubber for seeking
  - Speed adjustment slider

## WebSocket Protocol

Connect to `/ws/train/{client_id}` and send JSON commands:

```json
{"command": "start", "config": {"n_scouts": 5, "grid_size": 5, "episodes": 50}}
{"command": "pause"}
{"command": "resume"}
{"command": "set_speed", "speed": 2.0}
{"command": "stop"}
```

Server sends events:
- `scout_move` - Scout position update
- `episode_complete` - Scout finished episode
- `training_update` - Episode metrics
- `policy_update` - Learned policy grid
- `training_complete` - Training finished

## File Structure

```
web/
├── backend/                # Rust Axum server
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # Server entry point
│       ├── events.rs       # Event types
│       └── trainer.rs      # Training simulation
│
├── frontend/               # Rust Yew/WASM app
│   ├── Cargo.toml
│   ├── Trunk.toml
│   └── src/
│       ├── app.rs          # Root component
│       ├── components/     # UI components
│       ├── services/       # WebSocket client
│       └── types/          # Event structs
│
└── README.md
```
