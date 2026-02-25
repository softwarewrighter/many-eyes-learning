# Architecture

## Overview

Many-Eyes Learning is structured around the concept of **scouts** - independent exploratory agents that gather experience in sparse-reward environments. Their discoveries are aggregated to improve a shared learner.

## System Components

```
+------------------+     +------------------+     +------------------+
|     Scout 1      |     |     Scout 2      |     |     Scout N      |
| (exploration     |     | (exploration     |     | (exploration     |
|  strategy A)     |     |  strategy B)     |     |  strategy N)     |
+--------+---------+     +--------+---------+     +--------+---------+
         |                        |                        |
         v                        v                        v
+------------------------------------------------------------------------+
|                        Experience Buffer                                |
|  (stores trajectories, rewards, state transitions from all scouts)      |
+------------------------------------------------------------------------+
                                  |
                                  v
+------------------------------------------------------------------------+
|                       Aggregation Layer                                 |
|  (combines diverse experiences, weights by information value)           |
+------------------------------------------------------------------------+
                                  |
                                  v
+------------------------------------------------------------------------+
|                        Shared Learner                                   |
|  (policy/value network that learns from aggregated experience)          |
+------------------------------------------------------------------------+
```

## Component Details

### Scouts

Each scout is an exploratory agent with:

- **Exploration Strategy**: Defines how the scout explores (e.g., epsilon-greedy, curiosity-driven, count-based)
- **Local Policy**: May maintain its own policy or share the learner's policy with different exploration parameters
- **Trajectory Buffer**: Stores experiences during exploration runs

Scouts can be:
- **Homogeneous**: Same strategy with different random seeds
- **Heterogeneous**: Different exploration strategies for diversity

### Experience Buffer

Central storage for all scout experiences:

- **Trajectory Storage**: Complete episodes with states, actions, rewards
- **Metadata**: Scout ID, exploration strategy, timestamp
- **Priority Queue**: Optional prioritization based on novelty or reward

### Aggregation Layer

Combines scout experiences for learning:

- **Simple Aggregation**: Pool all experiences equally
- **Weighted Aggregation**: Weight by reward magnitude or novelty
- **Selective Aggregation**: Filter based on success criteria

### Shared Learner

The policy/value network that learns from aggregated experience:

- **Policy Network**: Maps states to actions
- **Value Network**: Estimates expected returns
- **Update Mechanism**: Standard RL algorithms (PPO, DQN, etc.)

## Execution Modes

### Sequential Mode

```
for scout in scouts:
    experiences = scout.explore(environment)
    buffer.add(experiences)

aggregated = aggregate(buffer)
learner.update(aggregated)
```

Benefits:
- Simple implementation
- Minimal resource requirements
- Reproducible results

### Parallel Mode

```
parallel:
    for scout in scouts:
        experiences = scout.explore(environment)
        buffer.add(experiences)

aggregated = aggregate(buffer)
learner.update(aggregated)
```

Benefits:
- Faster wall-clock time
- Better hardware utilization
- Same learning outcomes as sequential

## Data Flow

1. **Exploration Phase**
   - Each scout interacts with environment copy
   - Collects trajectory: (s, a, r, s', done)
   - Stores in local buffer

2. **Collection Phase**
   - Scout buffers merged into central buffer
   - Metadata attached for provenance

3. **Aggregation Phase**
   - Experiences combined according to strategy
   - Optional filtering or weighting applied

4. **Learning Phase**
   - Learner samples from aggregated buffer
   - Policy/value networks updated
   - Updated policy optionally distributed to scouts

## Environment Interface

```python
class Environment:
    def reset() -> State
    def step(action) -> (State, Reward, Done, Info)
    def render() -> None  # Optional
```

Environments are:
- **Copyable**: Each scout needs independent instance
- **Sparse-reward**: Primary use case (dense rewards work but are not the focus)
- **Deterministic or Stochastic**: Both supported

## Key Design Decisions

### Why Multiple Scouts?

In sparse-reward settings:
- Single explorer may never find reward
- Multiple explorers increase coverage
- Diversity of strategies prevents local optima

### Why Not Just Parallel Workers?

Standard parallel RL (A3C, IMPALA) uses identical workers. Many-eyes differs:
- Scouts can have different exploration strategies
- Focus is on information diversity, not throughput
- Sequential mode is first-class citizen

### Why Aggregate Before Learning?

Alternative: Learn from each scout independently, then ensemble.

Chosen approach benefits:
- Simpler architecture
- Single learner to maintain
- Cross-pollination of experiences

## Web Visualization Architecture

The web visualization provides real-time training observation:

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
│   - Replay controls                     │
│                                         │
│   API Endpoints                         │
│   - /ws/train/{client_id} (WebSocket)   │
│   - /api/health (REST)                  │
│   - StreamingTrainer                    │
└─────────────────────────────────────────┘
```

### Web Components

**Backend (Axum/Rust)**
- `StreamingTrainer`: Step-by-step training simulation with Q-learning
- WebSocket endpoint streams events in real-time
- Static file serving for frontend assets

**Frontend (Yew/WASM/Rust)**
- Grid component: SVG-based visualization of scouts and policy arrows
- Controls: Start/pause/stop with adjustable speed
- Metrics: Live success rate, loss, episode tracking
- Chart: High-DPI canvas-based learning curves
- Replay: Step through recorded training at adjustable speed (0.1x - 10x)

### Event Protocol

Server → Client:
- `ScoutMoveEvent`: Scout position, action, reward
- `EpisodeCompleteEvent`: Episode summary per scout
- `TrainingUpdateEvent`: Aggregated metrics per episode
- `PolicyUpdateEvent`: Current learned policy grid
- `TrainingCompleteEvent`: Final results

Client → Server:
- `start`: Begin training with config
- `pause`/`resume`: Control training
- `set_speed`: Adjust visualization speed
- `stop`: Terminate training

## File Organization

```
many-eyes-learning/
|-- src/
|   |-- scouts/           # Scout implementations
|   |   |-- base.py       # Scout interface
|   |   |-- epsilon.py    # Epsilon-greedy scout
|   |   |-- curious.py    # Curiosity-driven scout
|   |-- aggregation/      # Aggregation strategies
|   |   |-- simple.py     # Pool all experiences
|   |   |-- weighted.py   # Weight by novelty/reward
|   |-- learner/          # Shared learner
|   |   |-- policy.py     # Policy network
|   |   |-- value.py      # Value network
|   |-- buffer/           # Experience storage
|   |   |-- replay.py     # Replay buffer
|   |-- envs/             # Environment wrappers
|   |   |-- sparse.py     # Sparse-reward environments
|-- experiments/          # Reproducible experiments
|-- web/                  # Web visualization
|   |-- backend/          # Axum backend (Rust)
|   |   |-- src/
|   |       |-- main.rs      # Server entry point
|   |       |-- events.rs    # Event types
|   |       |-- trainer.rs   # Q-learning training simulation
|   |-- frontend/         # Yew/WASM frontend (Rust)
|       |-- src/
|           |-- app.rs       # Root component with reducer
|           |-- components/  # Grid, Controls, Chart, Replay
|           |-- services/    # WebSocket client
|           |-- types/       # Event structs
|-- docs/                 # Documentation
```

## Extension Points

### Custom Scouts

Implement `Scout` interface:
```python
class CustomScout(Scout):
    def explore(self, env, steps) -> List[Trajectory]
    def update_policy(self, policy) -> None
```

### Custom Aggregation

Implement `Aggregator` interface:
```python
class CustomAggregator(Aggregator):
    def aggregate(self, buffers) -> AggregatedBuffer
```

### Custom Environments

Wrap with standard interface:
```python
class CustomEnv(Environment):
    def reset(self) -> State
    def step(self, action) -> Tuple[State, Reward, Done, Info]
```

## Performance Considerations

### Memory

- Scout buffers: O(scouts * episode_length * state_size)
- Central buffer: O(total_experiences * state_size)
- Consider circular buffers for long-running experiments

### Computation

- Exploration: Linear in scouts (parallelizable)
- Aggregation: Linear in total experiences
- Learning: Depends on algorithm and batch size

### Scaling

- Scouts scale horizontally (more machines = more scouts)
- Learner is typically single-threaded (GPU-bound)
- Buffer may need distributed storage for large experiments
