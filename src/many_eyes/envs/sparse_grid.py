"""Sparse-reward grid world environment."""

from copy import deepcopy
from dataclasses import dataclass, field
from typing import Any

import numpy as np


@dataclass
class SparseGridWorld:
    """
    A simple grid world with sparse rewards.

    The agent starts at (0, 0) and must reach the goal at (size-1, size-1).
    Reward is +1 only when reaching the goal, 0 otherwise.

    Actions: 0=up, 1=right, 2=down, 3=left
    """

    size: int = 5
    walls: set[tuple[int, int]] = field(default_factory=set)
    max_steps: int = 100
    seed: int | None = None

    # Internal state
    _pos: tuple[int, int] = field(default=(0, 0), init=False)
    _steps: int = field(default=0, init=False)
    _rng: np.random.Generator = field(default=None, init=False)

    def __post_init__(self):
        """Initialize RNG."""
        self._rng = np.random.default_rng(self.seed)
        self._pos = (0, 0)
        self._steps = 0

    @property
    def state_shape(self) -> tuple[int, ...]:
        """State is flattened grid position as one-hot."""
        return (self.size * self.size,)

    @property
    def n_actions(self) -> int:
        """Four directions."""
        return 4

    @property
    def goal(self) -> tuple[int, int]:
        """Goal position."""
        return (self.size - 1, self.size - 1)

    def reset(self) -> np.ndarray:
        """Reset to start position."""
        self._pos = (0, 0)
        self._steps = 0
        return self._get_state()

    def step(self, action: int) -> tuple[np.ndarray, float, bool, dict[str, Any]]:
        """
        Take a step in the environment.

        Args:
            action: 0=up, 1=right, 2=down, 3=left

        Returns:
            (next_state, reward, done, info)
        """
        self._steps += 1

        # Compute next position
        row, col = self._pos
        if action == 0:  # up
            row = max(0, row - 1)
        elif action == 1:  # right
            col = min(self.size - 1, col + 1)
        elif action == 2:  # down
            row = min(self.size - 1, row + 1)
        elif action == 3:  # left
            col = max(0, col - 1)

        # Check for walls
        new_pos = (row, col)
        if new_pos not in self.walls:
            self._pos = new_pos

        # Check for goal
        reward = 1.0 if self._pos == self.goal else 0.0
        done = self._pos == self.goal or self._steps >= self.max_steps

        info = {
            "position": self._pos,
            "steps": self._steps,
            "reached_goal": self._pos == self.goal,
        }

        return self._get_state(), reward, done, info

    def _get_state(self) -> np.ndarray:
        """Convert position to one-hot state vector."""
        state = np.zeros(self.size * self.size, dtype=np.float32)
        idx = self._pos[0] * self.size + self._pos[1]
        state[idx] = 1.0
        return state

    def copy(self) -> "SparseGridWorld":
        """Return an independent copy."""
        new_env = SparseGridWorld(
            size=self.size,
            walls=deepcopy(self.walls),
            max_steps=self.max_steps,
            seed=None,  # New RNG for copy
        )
        new_env._pos = self._pos
        new_env._steps = self._steps
        return new_env

    def render(self) -> str:
        """Render grid as ASCII string."""
        lines = []
        for row in range(self.size):
            line = ""
            for col in range(self.size):
                pos = (row, col)
                if pos == self._pos:
                    line += "A "  # Agent
                elif pos == self.goal:
                    line += "G "  # Goal
                elif pos in self.walls:
                    line += "# "  # Wall
                else:
                    line += ". "  # Empty
            lines.append(line)
        return "\n".join(lines)

    @classmethod
    def with_obstacles(cls, size: int = 5, seed: int | None = None) -> "SparseGridWorld":
        """Create a grid with some random obstacles."""
        rng = np.random.default_rng(seed)

        # Place some walls, avoiding start and goal
        walls = set()
        n_walls = size  # Roughly size walls
        for _ in range(n_walls):
            row = rng.integers(0, size)
            col = rng.integers(0, size)
            pos = (row, col)
            # Don't block start or goal
            if pos != (0, 0) and pos != (size - 1, size - 1):
                walls.add(pos)

        return cls(size=size, walls=walls, seed=seed)
