# Many-Eyes Learning Web Visualization

Real-time visualization of multi-scout reinforcement learning training.

**Stack:** Rust + Yew/WASM (frontend) + Axum (backend)

## Quick Start

```bash
# Build and run on http://localhost:3200
./scripts/serve.sh
```

## Development

```bash
# Install dependencies
cargo install trunk wasm-bindgen-cli

# Development mode (hot reload)
./scripts/dev.sh
# Frontend: http://localhost:8080
# Backend:  http://localhost:3200
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
- **Learning Curves**: Canvas-based chart of training progress
- **Policy Visualization**: Arrow overlay showing learned policy

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
