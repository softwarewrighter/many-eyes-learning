"""Epsilon-greedy scout implementation."""

from dataclasses import dataclass, field

import numpy as np

from many_eyes.core import Trajectory, Transition
from many_eyes.protocols import Environment, Policy


@dataclass
class EpsilonGreedyScout:
    """
    A scout that follows a policy with epsilon-greedy exploration.

    With probability epsilon, takes a random action.
    Otherwise, follows the provided policy.
    """

    epsilon: float = 0.1
    scout_id: str = "epsilon_greedy"
    seed: int | None = None
    _rng: np.random.Generator = field(default=None, init=False)
    _policy: Policy | None = field(default=None, init=False)

    def __post_init__(self):
        self._rng = np.random.default_rng(self.seed)
        if not self.scout_id.startswith("epsilon"):
            self.scout_id = f"epsilon_{self.epsilon}_{self.scout_id}"

    @property
    def name(self) -> str:
        return self.scout_id

    def explore(self, env: Environment, steps: int) -> list[Trajectory]:
        """
        Explore with epsilon-greedy action selection.

        Args:
            env: Environment to explore
            steps: Maximum total steps across all episodes

        Returns:
            List of trajectories collected
        """
        trajectories = []
        total_steps = 0

        while total_steps < steps:
            trajectory = Trajectory(
                metadata={
                    "scout": self.name,
                    "strategy": "epsilon_greedy",
                    "epsilon": self.epsilon,
                }
            )
            state = env.reset()
            done = False

            while not done and total_steps < steps:
                action = self._select_action(state, env.n_actions)
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
        """Select action with epsilon-greedy strategy."""
        if self._rng.random() < self.epsilon:
            return self._rng.integers(0, n_actions)

        if self._policy is not None:
            return self._policy(state)

        # No policy set, fall back to random
        return self._rng.integers(0, n_actions)

    def set_policy(self, policy: Policy) -> None:
        """Update the policy used for greedy action selection."""
        self._policy = policy
