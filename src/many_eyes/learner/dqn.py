"""DQN Learner implementation."""

from dataclasses import dataclass, field
from typing import Callable

import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim

from many_eyes.buffer.replay import ReplayBuffer
from many_eyes.core import Trajectory


class QNetwork(nn.Module):
    """Simple Q-network for discrete actions."""

    def __init__(self, state_dim: int, n_actions: int, hidden_dim: int = 64):
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(state_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, n_actions),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.net(x)


@dataclass
class DQNLearner:
    """
    Deep Q-Network learner.

    Simple DQN with target network and replay buffer.
    """

    state_dim: int
    n_actions: int
    hidden_dim: int = 64
    learning_rate: float = 1e-3
    gamma: float = 0.99
    batch_size: int = 32
    target_update_freq: int = 100
    buffer_capacity: int = 100_000
    seed: int | None = None

    # Internal state
    _q_network: QNetwork = field(default=None, init=False)
    _target_network: QNetwork = field(default=None, init=False)
    _optimizer: optim.Optimizer = field(default=None, init=False)
    _buffer: ReplayBuffer = field(default=None, init=False)
    _update_count: int = field(default=0, init=False)
    _device: torch.device = field(default=None, init=False)

    def __post_init__(self):
        if self.seed is not None:
            torch.manual_seed(self.seed)

        self._device = torch.device("cuda" if torch.cuda.is_available() else "cpu")

        self._q_network = QNetwork(self.state_dim, self.n_actions, self.hidden_dim)
        self._q_network.to(self._device)

        self._target_network = QNetwork(self.state_dim, self.n_actions, self.hidden_dim)
        self._target_network.load_state_dict(self._q_network.state_dict())
        self._target_network.to(self._device)

        self._optimizer = optim.Adam(self._q_network.parameters(), lr=self.learning_rate)
        self._buffer = ReplayBuffer(capacity=self.buffer_capacity)

    def update(self, trajectories: list[Trajectory], steps: int) -> dict[str, float]:
        """
        Update Q-network from trajectories.

        Args:
            trajectories: Trajectories to learn from
            steps: Number of gradient steps to take

        Returns:
            Training metrics
        """
        # Add trajectories to buffer
        self._buffer.add_trajectories(trajectories)

        if len(self._buffer) < self.batch_size:
            return {"loss": 0.0, "buffer_size": len(self._buffer)}

        total_loss = 0.0
        for _ in range(steps):
            loss = self._train_step()
            total_loss += loss

            self._update_count += 1
            if self._update_count % self.target_update_freq == 0:
                self._target_network.load_state_dict(self._q_network.state_dict())

        return {
            "loss": total_loss / steps,
            "buffer_size": len(self._buffer),
            "update_count": self._update_count,
        }

    def _train_step(self) -> float:
        """Single training step."""
        batch = self._buffer.sample(self.batch_size)

        states = torch.tensor(batch["states"], dtype=torch.float32, device=self._device)
        actions = torch.tensor(batch["actions"], dtype=torch.long, device=self._device)
        rewards = torch.tensor(batch["rewards"], dtype=torch.float32, device=self._device)
        next_states = torch.tensor(batch["next_states"], dtype=torch.float32, device=self._device)
        dones = torch.tensor(batch["dones"], dtype=torch.float32, device=self._device)

        # Current Q values
        q_values = self._q_network(states)
        q_values = q_values.gather(1, actions.unsqueeze(1)).squeeze(1)

        # Target Q values
        with torch.no_grad():
            next_q_values = self._target_network(next_states)
            max_next_q = next_q_values.max(1)[0]
            targets = rewards + self.gamma * max_next_q * (1 - dones)

        # Loss and update
        loss = nn.functional.mse_loss(q_values, targets)

        self._optimizer.zero_grad()
        loss.backward()
        self._optimizer.step()

        return loss.item()

    def get_policy(self) -> Callable[[np.ndarray], int]:
        """Return greedy policy function."""

        def policy(state: np.ndarray) -> int:
            with torch.no_grad():
                state_tensor = torch.tensor(state, dtype=torch.float32, device=self._device)
                q_values = self._q_network(state_tensor)
                return int(q_values.argmax().item())

        return policy
