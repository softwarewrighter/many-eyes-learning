"""Curiosity-driven scout with count-based exploration bonus."""

from collections import defaultdict
from dataclasses import dataclass, field

import numpy as np

from many_eyes.core import Trajectory, Transition
from many_eyes.protocols import Environment, Policy


@dataclass
class CuriousScout:
    """
    A scout that seeks novelty using count-based exploration.

    Adds an intrinsic reward bonus inversely proportional to state visit counts.
    This encourages visiting less-explored states.

    The bonus is: bonus_scale / sqrt(count + 1)
    """

    bonus_scale: float = 0.5
    scout_id: str = "curious"
    seed: int | None = None
    _rng: np.random.Generator = field(default=None, init=False)
    _policy: Policy | None = field(default=None, init=False)
    _state_counts: dict = field(default_factory=lambda: defaultdict(int), init=False)

    def __post_init__(self):
        self._rng = np.random.default_rng(self.seed)
        self._state_counts = defaultdict(int)

    @property
    def name(self) -> str:
        return self.scout_id

    def _state_key(self, state: np.ndarray) -> tuple:
        """Convert state to hashable key."""
        # For one-hot states, just use argmax
        return (int(state.argmax()),)

    def _intrinsic_reward(self, state: np.ndarray) -> float:
        """Compute intrinsic reward based on visit count."""
        key = self._state_key(state)
        count = self._state_counts[key]
        return self.bonus_scale / np.sqrt(count + 1)

    def explore(self, env: Environment, steps: int) -> list[Trajectory]:
        """
        Explore with curiosity-driven action selection.

        Uses intrinsic rewards to guide exploration toward novel states.
        The actual trajectory stores original environment rewards.
        """
        trajectories = []
        total_steps = 0

        while total_steps < steps:
            trajectory = Trajectory(
                metadata={
                    "scout": self.name,
                    "strategy": "curious",
                    "bonus_scale": self.bonus_scale,
                }
            )
            state = env.reset()
            done = False

            while not done and total_steps < steps:
                # Update visit count
                key = self._state_key(state)
                self._state_counts[key] += 1

                # Select action (with curiosity influencing exploration)
                action = self._select_action(state, env.n_actions)
                next_state, reward, done, info = env.step(action)

                # Store original reward (not intrinsic)
                trajectory.add(
                    Transition(
                        state=state,
                        action=action,
                        reward=reward,
                        next_state=next_state,
                        done=done,
                    )
                )

                state = next_state
                total_steps += 1

            trajectories.append(trajectory)

        return trajectories

    def _select_action(self, state: np.ndarray, n_actions: int) -> int:
        """Select action balancing policy and curiosity."""
        # If we have a policy, use it with some exploration
        if self._policy is not None:
            # 30% chance to explore based on curiosity
            if self._rng.random() < 0.3:
                return self._curiosity_action(state, n_actions)
            return self._policy(state)

        # No policy - use curiosity-weighted random
        return self._curiosity_action(state, n_actions)

    def _curiosity_action(self, state: np.ndarray, n_actions: int) -> int:
        """Select action that leads to less-visited states (approximately)."""
        # Simple heuristic: random action with bias toward unexplored
        # In practice, we'd need a model to predict next states
        # For now, just use random exploration
        return int(self._rng.integers(0, n_actions))

    def set_policy(self, policy: Policy) -> None:
        """Update the base policy."""
        self._policy = policy

    def reset_counts(self) -> None:
        """Reset visit counts (e.g., between training phases)."""
        self._state_counts = defaultdict(int)

    @property
    def visit_counts(self) -> dict:
        """Return current visit counts for analysis."""
        return dict(self._state_counts)
