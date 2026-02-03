"""Base scout implementations."""

from dataclasses import dataclass, field

import numpy as np

from many_eyes.core import Trajectory, Transition
from many_eyes.protocols import Environment, Policy


@dataclass
class RandomScout:
    """
    A scout that explores purely randomly.

    Useful as a baseline and for environments where
    random exploration can find rewards.
    """

    scout_id: str = "random"
    seed: int | None = None
    _rng: np.random.Generator = field(default=None, init=False)
    _policy: Policy | None = field(default=None, init=False)

    def __post_init__(self):
        self._rng = np.random.default_rng(self.seed)

    @property
    def name(self) -> str:
        return self.scout_id

    def explore(self, env: Environment, steps: int) -> list[Trajectory]:
        """
        Explore environment with random actions.

        Args:
            env: Environment to explore
            steps: Maximum total steps across all episodes

        Returns:
            List of trajectories collected
        """
        trajectories = []
        total_steps = 0

        while total_steps < steps:
            trajectory = Trajectory(metadata={"scout": self.name, "strategy": "random"})
            state = env.reset()
            done = False

            while not done and total_steps < steps:
                action = self._rng.integers(0, env.n_actions)
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

    def set_policy(self, policy: Policy) -> None:
        """Random scout ignores policy updates."""
        self._policy = policy
