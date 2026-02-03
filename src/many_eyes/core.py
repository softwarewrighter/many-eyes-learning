"""Core data structures for Many-Eyes Learning."""

from dataclasses import dataclass, field
from typing import Any

import numpy as np


@dataclass
class Transition:
    """A single environment transition (s, a, r, s', done)."""

    state: np.ndarray
    action: int
    reward: float
    next_state: np.ndarray
    done: bool

    def __post_init__(self):
        """Ensure arrays are numpy arrays."""
        if not isinstance(self.state, np.ndarray):
            self.state = np.array(self.state)
        if not isinstance(self.next_state, np.ndarray):
            self.next_state = np.array(self.next_state)


@dataclass
class Trajectory:
    """A sequence of transitions from a single episode."""

    transitions: list[Transition] = field(default_factory=list)
    metadata: dict[str, Any] = field(default_factory=dict)

    def add(self, transition: Transition) -> None:
        """Add a transition to the trajectory."""
        self.transitions.append(transition)

    def __len__(self) -> int:
        return len(self.transitions)

    def __iter__(self):
        return iter(self.transitions)

    @property
    def total_reward(self) -> float:
        """Sum of rewards in this trajectory."""
        return sum(t.reward for t in self.transitions)

    @property
    def states(self) -> list[np.ndarray]:
        """All states in order."""
        return [t.state for t in self.transitions]

    @property
    def final_state(self) -> np.ndarray | None:
        """The final next_state, or None if empty."""
        if self.transitions:
            return self.transitions[-1].next_state
        return None

    @property
    def reached_goal(self) -> bool:
        """Whether this trajectory ended with positive reward."""
        if self.transitions:
            return self.transitions[-1].reward > 0
        return False
