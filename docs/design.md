# Design Document

## Design Philosophy

This project prioritizes **clarity and intuition** over performance. Every design decision should make the core insight easier to understand:

> Better exploration leads to better information leads to better learning.

## Core Abstractions

### Scout

A scout is an exploratory agent. The key insight is that scouts are **information gatherers**, not independent learners.

```python
class Scout(Protocol):
    """An exploratory agent that gathers experience."""

    def explore(self, env: Environment, steps: int) -> List[Trajectory]:
        """Run exploration and return collected trajectories."""
        ...

    def update_policy(self, policy: Policy) -> None:
        """Optionally update exploration policy from shared learner."""
        ...
```

Design decisions:
- Scouts return trajectories, not policies
- Scouts may or may not use the shared policy
- Exploration strategy is encapsulated in the scout

### Trajectory

A trajectory is a sequence of transitions from a single episode.

```python
@dataclass
class Transition:
    state: np.ndarray
    action: int
    reward: float
    next_state: np.ndarray
    done: bool

@dataclass
class Trajectory:
    transitions: List[Transition]
    metadata: dict  # scout_id, strategy, timestamp, etc.
```

Design decisions:
- Include metadata for provenance tracking
- Store complete episodes, not fragments
- Keep transitions simple (SARSD)

### Aggregator

Combines experiences from multiple scouts.

```python
class Aggregator(Protocol):
    """Combines scout experiences for learning."""

    def aggregate(self, trajectories: List[Trajectory]) -> ReplayBuffer:
        """Combine trajectories into unified buffer."""
        ...
```

Design decisions:
- Output is a standard replay buffer
- Aggregation happens once per learning cycle
- Strategy is pluggable

### Learner

The shared learner that updates from aggregated experience.

```python
class Learner(Protocol):
    """Learns from aggregated experience."""

    def update(self, buffer: ReplayBuffer, steps: int) -> dict:
        """Update policy/value networks from buffer."""
        ...

    def get_policy(self) -> Policy:
        """Return current policy for scouts."""
        ...
```

Design decisions:
- Standard RL learner interface
- Returns metrics for logging
- Policy extraction for scout updates

## Exploration Strategies

### Epsilon-Greedy

Simplest strategy: random action with probability epsilon.

```python
class EpsilonGreedyScout(Scout):
    def __init__(self, epsilon: float = 0.1):
        self.epsilon = epsilon

    def select_action(self, state, policy):
        if random.random() < self.epsilon:
            return random_action()
        return policy(state)
```

Configuration variations:
- Fixed epsilon
- Decaying epsilon
- Different epsilon per scout

### Curiosity-Driven

Bonus reward for novel states.

```python
class CuriousScout(Scout):
    def __init__(self, bonus_scale: float = 1.0):
        self.state_counts = defaultdict(int)
        self.bonus_scale = bonus_scale

    def intrinsic_reward(self, state):
        count = self.state_counts[state]
        return self.bonus_scale / sqrt(count + 1)
```

Design notes:
- Simple count-based novelty (not learned)
- Bonus added to trajectory rewards
- Each scout has independent counts

### Optimistic

Initialize Q-values optimistically to encourage exploration.

```python
class OptimisticScout(Scout):
    def __init__(self, optimism: float = 10.0):
        self.optimism = optimism

    def initial_q_value(self):
        return self.optimism  # Instead of 0
```

## Aggregation Strategies

### Simple Pooling

All experiences treated equally.

```python
class SimpleAggregator(Aggregator):
    def aggregate(self, trajectories):
        buffer = ReplayBuffer()
        for traj in trajectories:
            buffer.add_trajectory(traj)
        return buffer
```

Use when: No prior about which scout has better experience.

### Reward-Weighted

Weight experiences by reward magnitude.

```python
class RewardWeightedAggregator(Aggregator):
    def aggregate(self, trajectories):
        buffer = PrioritizedReplayBuffer()
        for traj in trajectories:
            priority = sum(t.reward for t in traj.transitions)
            buffer.add_trajectory(traj, priority)
        return buffer
```

Use when: Want to emphasize successful trajectories.

### Novelty-Weighted

Weight experiences by novelty of states visited.

```python
class NoveltyWeightedAggregator(Aggregator):
    def __init__(self):
        self.state_counts = defaultdict(int)

    def aggregate(self, trajectories):
        buffer = PrioritizedReplayBuffer()
        for traj in trajectories:
            novelty = self._compute_novelty(traj)
            buffer.add_trajectory(traj, novelty)
        return buffer
```

Use when: Want to emphasize exploration of new areas.

## Environment Design

### Sparse Grid World

Primary demo environment.

```
+---+---+---+---+---+
| S |   |   |   |   |
+---+---+---+---+---+
|   | # |   | # |   |
+---+---+---+---+---+
|   |   |   |   |   |
+---+---+---+---+---+
|   | # |   | # |   |
+---+---+---+---+---+
|   |   |   |   | G |
+---+---+---+---+---+

S = Start, G = Goal (reward +1), # = Wall
All other transitions: reward 0
```

Design decisions:
- Small enough to visualize
- Sparse enough to demonstrate the problem
- Customizable size and wall placement

### Environment Interface

```python
class Environment(Protocol):
    def reset(self) -> np.ndarray:
        """Reset and return initial state."""
        ...

    def step(self, action: int) -> Tuple[np.ndarray, float, bool, dict]:
        """Take action, return (next_state, reward, done, info)."""
        ...

    def copy(self) -> 'Environment':
        """Return independent copy for parallel scouts."""
        ...
```

## Visualization Design

### Coverage Map

Shows which states each scout visited.

```
Scout 1    Scout 2    Scout 3    Combined
+-----+    +-----+    +-----+    +-----+
|X X  |    |  X X|    |X   X|    |X X X|
|X    |    |    X|    |  X  |    |X X X|
|X X  |    |X X  |    |  X X|    |X X X|
+-----+    +-----+    +-----+    +-----+
```

### Learning Curves

Compare single explorer vs. many-eyes.

```
Reward
  ^
  |         Many-eyes (N=5)
  |        ___________
  |       /
  |      /     Single
  |     /     ________
  |    /     /
  |___/     /
  +------------------------> Episodes
```

### Exploration Animation

Reconstruct exploration as video.

- Frame per timestep
- Different colors for different scouts
- Trail showing recent history

### Web Visualization

Real-time browser-based training visualization.

```
┌─────────────────────────────────────────────────────────────┐
│  Many-Eyes Learning                    [Connected]          │
├─────────────────────────────────────────────────────────────┤
│                              │  Controls                    │
│  ┌───────────────────────┐   │  [Start] [Pause] [Stop]      │
│  │  Grid Visualization   │   │  Speed: [====○===] 2x        │
│  │  (SVG with scouts)    │   │                              │
│  │                       │   │  Metrics                     │
│  │  0→ 1→ 2→ G           │   │  Episode: 45/100             │
│  │  ↓     ↑              │   │  Success: 60%                │
│  │  3→ 4→ 5↗             │   │  Loss: 0.0023                │
│  │                       │   │                              │
│  └───────────────────────┘   │  Scouts                      │
│                              │  ● Scout 0 (Random)    +12.3 │
│  ┌───────────────────────┐   │  ● Scout 1 (ε=0.14)   +8.7  │
│  │  Learning Curves      │   │  ● Scout 2 (ε=0.22)   +15.2 │
│  │  (Canvas chart)       │   │                              │
│  └───────────────────────┘   │                              │
└─────────────────────────────────────────────────────────────┘
```

Technology choices:
- **Backend**: FastAPI with WebSocket for real-time events
- **Frontend**: Yew/WASM for high-performance rendering
- **Protocol**: JSON events over WebSocket

Event types:
- `ScoutMoveEvent`: Position updates for animation
- `TrainingUpdateEvent`: Metrics per episode
- `PolicyUpdateEvent`: Arrow overlay updates

## Configuration

### Experiment Config

```yaml
experiment:
  name: "sparse_grid_5x5"
  seed: 42

scouts:
  count: 5
  strategies:
    - type: "epsilon_greedy"
      epsilon: 0.3
    - type: "epsilon_greedy"
      epsilon: 0.1
    - type: "curious"
      bonus_scale: 0.5
    - type: "optimistic"
      optimism: 10.0
    - type: "random"

aggregation:
  type: "simple"

learner:
  algorithm: "dqn"
  learning_rate: 0.001
  batch_size: 32

environment:
  type: "sparse_grid"
  size: 5

training:
  episodes: 1000
  steps_per_scout: 100
```

## Error Handling

### Scout Failures

If a scout crashes:
- Log the error
- Continue with other scouts
- Do not retry failed scout

### Environment Errors

If environment fails:
- Terminate affected scout's episode
- Continue with other scouts
- Log for debugging

### Learning Errors

If learner update fails:
- Stop training
- Save checkpoint
- Report error clearly

## Testing Strategy

### Unit Tests

- Scout action selection
- Buffer operations
- Aggregation correctness

### Integration Tests

- Full training loop (few episodes)
- Sequential vs parallel consistency
- Checkpoint save/load

### Visual Tests

- Coverage map generation
- Learning curve plotting
- Animation creation

## Extension Guidelines

### Adding a Scout Strategy

1. Implement `Scout` protocol
2. Add to strategy registry
3. Add configuration option
4. Write unit tests
5. Document in this file

### Adding an Aggregation Strategy

1. Implement `Aggregator` protocol
2. Add to aggregator registry
3. Add configuration option
4. Write unit tests
5. Document in this file

### Adding an Environment

1. Implement `Environment` protocol
2. Ensure `copy()` works correctly
3. Add configuration option
4. Write visual test
5. Document in this file

## Trade-offs and Alternatives

### Why Not Actor-Learner Separation?

Alternatives considered:
- IMPALA-style: Actors send to learner asynchronously
- Ape-X style: Distributed prioritized replay

Chosen approach:
- Synchronous batch updates
- Simpler to understand
- No distributed systems complexity

### Why Not Learned Exploration?

Alternatives considered:
- Learned curiosity (ICM, RND)
- Meta-learned exploration

Chosen approach:
- Hand-designed exploration strategies
- Easier to understand and debug
- Focus on aggregation, not exploration algorithms

### Why Single Learner?

Alternatives considered:
- Ensemble of learners
- Population-based training

Chosen approach:
- Single shared learner
- Simpler architecture
- Focus on exploration diversity, not learner diversity
