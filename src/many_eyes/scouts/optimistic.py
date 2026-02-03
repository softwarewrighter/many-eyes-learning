"""Optimistic scout with optimistic action value initialization."""

from collections import defaultdict
from dataclasses import dataclass, field

import numpy as np

from many_eyes.core import Trajectory, Transition
from many_eyes.protocols import Environment, Policy


@dataclass
class OptimisticScout:
    """
    A scout that uses optimistic initialization for exploration.

    Maintains optimistic Q-value estimates that start high and decay
    toward actual values. This encourages trying all actions initially.

    Uses UCB-like action selection: Q(s,a) + exploration_bonus
    """

    optimism: float = 1.0  # Initial optimistic value
    decay: float = 0.95  # How fast optimism decays
    scout_id: str = "optimistic"
    seed: int | None = None
    _rng: np.random.Generator = field(default=None, init=False)
    _policy: Policy | None = field(default=None, init=False)
    _action_counts: dict = field(default_factory=lambda: defaultdict(int), init=False)
    _optimistic_values: dict = field(default_factory=dict, init=False)

    def __post_init__(self):
        self._rng = np.random.default_rng(self.seed)
        self._action_counts = defaultdict(int)
        self._optimistic_values = {}

    @property
    def name(self) -> str:
        return self.scout_id

    def _state_action_key(self, state: np.ndarray, action: int) -> tuple:
        """Convert state-action to hashable key."""
        return (int(state.argmax()), action)

    def _get_optimistic_bonus(self, state: np.ndarray, action: int) -> float:
        """Get optimistic bonus for state-action pair."""
        key = self._state_action_key(state, action)
        count = self._action_counts[key]

        # Bonus decays with visits
        bonus = self.optimism * (self.decay**count)
        return bonus

    def explore(self, env: Environment, steps: int) -> list[Trajectory]:
        """
        Explore with optimistic action selection.

        Prefers actions that haven't been tried much in each state.
        """
        trajectories = []
        total_steps = 0

        while total_steps < steps:
            trajectory = Trajectory(
                metadata={
                    "scout": self.name,
                    "strategy": "optimistic",
                    "optimism": self.optimism,
                }
            )
            state = env.reset()
            done = False

            while not done and total_steps < steps:
                action = self._select_action(state, env.n_actions)

                # Update count
                key = self._state_action_key(state, action)
                self._action_counts[key] += 1

                next_state, reward, done, info = env.step(action)

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
        """Select action using optimistic values."""
        if self._policy is None:
            # No policy - select based purely on optimistic bonus
            bonuses = [self._get_optimistic_bonus(state, a) for a in range(n_actions)]
            # Add small noise for tie-breaking
            bonuses = [b + self._rng.random() * 0.01 for b in bonuses]
            return int(np.argmax(bonuses))

        # With policy - combine policy value with optimistic bonus
        # Small chance to use pure optimism
        if self._rng.random() < 0.2:
            bonuses = [self._get_optimistic_bonus(state, a) for a in range(n_actions)]
            bonuses = [b + self._rng.random() * 0.01 for b in bonuses]
            return int(np.argmax(bonuses))

        return self._policy(state)

    def set_policy(self, policy: Policy) -> None:
        """Update the base policy."""
        self._policy = policy

    def reset_counts(self) -> None:
        """Reset action counts."""
        self._action_counts = defaultdict(int)
