"""Protocol definitions for Many-Eyes Learning components."""

from typing import Any, Protocol

import numpy as np

from many_eyes.core import Trajectory


class Environment(Protocol):
    """Protocol for environments that scouts explore."""

    @property
    def state_shape(self) -> tuple[int, ...]:
        """Shape of state observations."""
        ...

    @property
    def n_actions(self) -> int:
        """Number of discrete actions."""
        ...

    def reset(self) -> np.ndarray:
        """Reset environment and return initial state."""
        ...

    def step(self, action: int) -> tuple[np.ndarray, float, bool, dict[str, Any]]:
        """Take action, return (next_state, reward, done, info)."""
        ...

    def copy(self) -> "Environment":
        """Return an independent copy of this environment."""
        ...


class Policy(Protocol):
    """Protocol for policies that map states to actions."""

    def __call__(self, state: np.ndarray) -> int:
        """Select action for given state."""
        ...


class Scout(Protocol):
    """Protocol for exploratory agents that gather experience."""

    @property
    def name(self) -> str:
        """Identifier for this scout."""
        ...

    def explore(self, env: Environment, steps: int) -> list[Trajectory]:
        """
        Run exploration and return collected trajectories.

        Args:
            env: Environment to explore
            steps: Maximum steps to take

        Returns:
            List of trajectories (episodes) collected
        """
        ...

    def set_policy(self, policy: Policy) -> None:
        """Update the base policy used for exploration."""
        ...


class Aggregator(Protocol):
    """Protocol for combining scout experiences."""

    def aggregate(self, trajectories: list[Trajectory]) -> list[Trajectory]:
        """
        Combine trajectories from multiple scouts.

        Args:
            trajectories: All trajectories from all scouts

        Returns:
            Processed trajectories ready for learning
        """
        ...


class Learner(Protocol):
    """Protocol for the shared learner."""

    def update(self, trajectories: list[Trajectory], steps: int) -> dict[str, float]:
        """
        Update from aggregated experience.

        Args:
            trajectories: Trajectories to learn from
            steps: Number of gradient steps

        Returns:
            Dictionary of training metrics
        """
        ...

    def get_policy(self) -> Policy:
        """Return current policy for scouts to use."""
        ...
