"""Streaming trainer that emits events during training."""

import asyncio
from dataclasses import dataclass, field

import numpy as np

from many_eyes.core import Trajectory, Transition
from many_eyes.envs.sparse_grid import SparseGridWorld
from many_eyes.learner.dqn import DQNLearner
from many_eyes.protocols import Scout
from many_eyes.scouts.base import RandomScout
from many_eyes.scouts.epsilon import EpsilonGreedyScout

from ..models.events import (
    EpisodeCompleteEvent,
    ErrorEvent,
    PolicyUpdateEvent,
    ScoutMoveEvent,
    TrainingCompleteEvent,
    TrainingConfig,
    TrainingUpdateEvent,
)


@dataclass
class StreamingTrainer:
    """
    Trainer that yields events during training for real-time visualization.

    This implements a step-by-step training loop that emits events
    for each scout action, enabling real-time visualization.
    """

    config: TrainingConfig
    speed: float = 1.0
    paused: bool = False
    stopped: bool = False

    # Internal state
    _env: SparseGridWorld = field(default=None, init=False)
    _scouts: list[Scout] = field(default_factory=list, init=False)
    _learner: DQNLearner = field(default=None, init=False)
    _history: dict[str, list[float]] = field(default_factory=dict, init=False)

    def __post_init__(self):
        """Initialize environment, scouts, and learner."""
        # Create environment
        if self.config.with_obstacles:
            self._env = SparseGridWorld.with_obstacles(
                size=self.config.grid_size, seed=self.config.seed
            )
        else:
            self._env = SparseGridWorld(
                size=self.config.grid_size, seed=self.config.seed
            )

        # Create scouts with their own RNGs for step-by-step control
        self._scouts = []
        self._scout_rngs = []
        for i in range(self.config.n_scouts):
            scout_seed = self.config.seed + i if self.config.seed is not None else None
            rng = np.random.default_rng(scout_seed)
            self._scout_rngs.append(rng)

            if i == 0:
                self._scouts.append(
                    RandomScout(scout_id=f"random_{i}", seed=scout_seed)
                )
            else:
                epsilon = 0.1 + 0.2 * (i / self.config.n_scouts)
                self._scouts.append(
                    EpsilonGreedyScout(
                        epsilon=epsilon,
                        scout_id=f"eps_{epsilon:.2f}_{i}",
                        seed=scout_seed,
                    )
                )

        # Create learner
        self._learner = DQNLearner(
            state_dim=self._env.state_shape[0],
            n_actions=self._env.n_actions,
            seed=self.config.seed,
        )

        # Initialize history
        self._history = {
            "episode_rewards": [],
            "success_rates": [],
            "losses": [],
        }

    def pause(self) -> None:
        """Pause training."""
        self.paused = True

    def resume(self) -> None:
        """Resume training."""
        self.paused = False

    def stop(self) -> None:
        """Stop training."""
        self.stopped = True

    def set_speed(self, speed: float) -> None:
        """Set training speed multiplier."""
        self.speed = max(0.1, min(10.0, speed))

    async def _wait_for_resume(self) -> bool:
        """Wait while paused. Returns False if stopped."""
        while self.paused and not self.stopped:
            await asyncio.sleep(0.1)
        return not self.stopped

    def _state_to_position(self, state: np.ndarray) -> tuple[int, int]:
        """Convert one-hot state to grid position."""
        idx = int(np.argmax(state))
        row = idx // self.config.grid_size
        col = idx % self.config.grid_size
        return (row, col)

    def _get_policy_grid(self) -> list[list[int]]:
        """Get current policy as a grid of actions."""
        policy = self._learner.get_policy()
        grid_size = self.config.grid_size
        policy_grid = []
        for row in range(grid_size):
            row_actions = []
            for col in range(grid_size):
                state = np.zeros(grid_size * grid_size, dtype=np.float32)
                state[row * grid_size + col] = 1.0
                action = policy(state)
                row_actions.append(int(action))
            policy_grid.append(row_actions)
        return policy_grid

    def _select_action(self, scout_idx: int, state: np.ndarray) -> int:
        """Select action for a scout, matching the scout's strategy."""
        scout = self._scouts[scout_idx]
        rng = self._scout_rngs[scout_idx]

        if isinstance(scout, RandomScout):
            return int(rng.integers(0, self._env.n_actions))
        elif isinstance(scout, EpsilonGreedyScout):
            if rng.random() < scout.epsilon or scout._policy is None:
                return int(rng.integers(0, self._env.n_actions))
            else:
                return int(scout._policy(state))
        else:
            return int(rng.integers(0, self._env.n_actions))

    async def train(self):
        """
        Run training and yield events.

        This is an async generator that yields events during training,
        allowing real-time visualization.
        """
        try:
            for episode in range(self.config.episodes):
                if self.stopped:
                    break

                # Wait if paused
                if not await self._wait_for_resume():
                    break

                # Exploration phase - each scout explores with event emission
                all_trajectories: list[Trajectory] = []

                for scout_idx, scout in enumerate(self._scouts):
                    if self.stopped:
                        break

                    env_copy = self._env.copy()
                    state = env_copy.reset()
                    trajectory = Trajectory(
                        metadata={"scout_id": scout.scout_id, "episode": episode}
                    )
                    step = 0

                    while step < self.config.steps_per_episode:
                        if self.stopped:
                            break

                        # Wait if paused
                        if not await self._wait_for_resume():
                            break

                        # Select action using scout's strategy
                        action = self._select_action(scout_idx, state)

                        # Take step
                        next_state, reward, done, info = env_copy.step(action)

                        # Add to trajectory
                        transition = Transition(
                            state=state,
                            action=action,
                            reward=reward,
                            next_state=next_state,
                            done=done,
                        )
                        trajectory.add(transition)

                        # Emit move event
                        position = self._state_to_position(next_state)
                        yield ScoutMoveEvent(
                            scout_id=scout.scout_id,
                            scout_index=scout_idx,
                            position=position,
                            action=action,
                            reward=reward,
                            done=done,
                            step=step,
                        )

                        # Delay based on speed
                        await asyncio.sleep(0.05 / self.speed)

                        state = next_state
                        step += 1

                        if done:
                            break

                    # Episode complete for this scout
                    if not self.stopped:
                        yield EpisodeCompleteEvent(
                            scout_id=scout.scout_id,
                            scout_index=scout_idx,
                            reached_goal=trajectory.reached_goal,
                            total_reward=trajectory.total_reward,
                            steps=len(trajectory),
                        )

                    all_trajectories.append(trajectory)

                if self.stopped:
                    break

                # Learning phase
                metrics = self._learner.update(all_trajectories, steps=10)

                # Track metrics
                episode_reward = sum(t.total_reward for t in all_trajectories)
                successes = sum(1 for t in all_trajectories if t.reached_goal)
                success_rate = (
                    successes / len(all_trajectories) if all_trajectories else 0
                )

                self._history["episode_rewards"].append(episode_reward)
                self._history["success_rates"].append(success_rate)
                self._history["losses"].append(metrics.get("loss", 0))

                # Emit training update
                yield TrainingUpdateEvent(
                    episode=episode + 1,
                    total_episodes=self.config.episodes,
                    success_rate=success_rate,
                    loss=metrics.get("loss", 0),
                    episode_reward=episode_reward,
                )

                # Update scout policies periodically
                if (episode + 1) % 10 == 0:
                    policy = self._learner.get_policy()
                    for scout in self._scouts:
                        scout.set_policy(policy)

                    # Emit policy update
                    yield PolicyUpdateEvent(policy=self._get_policy_grid())

            # Training complete
            final_success_rate = (
                float(np.mean(self._history["success_rates"][-10:]))
                if self._history["success_rates"]
                else 0.0
            )
            yield TrainingCompleteEvent(
                final_success_rate=final_success_rate,
                history=self._history,
            )

        except Exception as e:
            yield ErrorEvent(message=str(e))
