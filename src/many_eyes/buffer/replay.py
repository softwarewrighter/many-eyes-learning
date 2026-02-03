"""Replay buffer for storing and sampling experiences."""

from dataclasses import dataclass, field

import numpy as np

from many_eyes.core import Trajectory, Transition


@dataclass
class ReplayBuffer:
    """
    Simple replay buffer for storing transitions.

    Supports adding trajectories and sampling random batches.
    """

    capacity: int = 100_000
    _transitions: list[Transition] = field(default_factory=list, init=False)
    _position: int = field(default=0, init=False)

    def add_transition(self, transition: Transition) -> None:
        """Add a single transition to the buffer."""
        if len(self._transitions) < self.capacity:
            self._transitions.append(transition)
        else:
            self._transitions[self._position] = transition

        self._position = (self._position + 1) % self.capacity

    def add_trajectory(self, trajectory: Trajectory) -> None:
        """Add all transitions from a trajectory."""
        for transition in trajectory:
            self.add_transition(transition)

    def add_trajectories(self, trajectories: list[Trajectory]) -> None:
        """Add all transitions from multiple trajectories."""
        for trajectory in trajectories:
            self.add_trajectory(trajectory)

    def sample(self, batch_size: int) -> dict[str, np.ndarray]:
        """
        Sample a random batch of transitions.

        Returns:
            Dictionary with keys: states, actions, rewards, next_states, dones
        """
        indices = np.random.randint(0, len(self._transitions), size=batch_size)
        transitions = [self._transitions[i] for i in indices]

        return {
            "states": np.array([t.state for t in transitions]),
            "actions": np.array([t.action for t in transitions]),
            "rewards": np.array([t.reward for t in transitions], dtype=np.float32),
            "next_states": np.array([t.next_state for t in transitions]),
            "dones": np.array([t.done for t in transitions], dtype=np.float32),
        }

    def __len__(self) -> int:
        return len(self._transitions)

    def clear(self) -> None:
        """Clear all transitions."""
        self._transitions = []
        self._position = 0
