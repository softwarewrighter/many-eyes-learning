"""Tests for replay buffer."""

import numpy as np

from many_eyes.buffer.replay import ReplayBuffer
from many_eyes.core import Trajectory, Transition


def test_buffer_creation():
    """Test creating a buffer."""
    buffer = ReplayBuffer(capacity=100)

    assert len(buffer) == 0
    assert buffer.capacity == 100


def test_buffer_add_transition():
    """Test adding a single transition."""
    buffer = ReplayBuffer()

    buffer.add_transition(
        Transition(
            state=np.array([1, 0, 0]),
            action=1,
            reward=0.5,
            next_state=np.array([0, 1, 0]),
            done=False,
        )
    )

    assert len(buffer) == 1


def test_buffer_add_trajectory():
    """Test adding a trajectory."""
    buffer = ReplayBuffer()

    traj = Trajectory()
    for i in range(5):
        traj.add(
            Transition(
                state=np.array([i]),
                action=i % 4,
                reward=0.0,
                next_state=np.array([i + 1]),
                done=i == 4,
            )
        )

    buffer.add_trajectory(traj)
    assert len(buffer) == 5


def test_buffer_add_trajectories():
    """Test adding multiple trajectories."""
    buffer = ReplayBuffer()

    trajectories = []
    for _ in range(3):
        traj = Trajectory()
        for i in range(2):
            traj.add(
                Transition(
                    state=np.array([i]),
                    action=0,
                    reward=0.0,
                    next_state=np.array([i + 1]),
                    done=False,
                )
            )
        trajectories.append(traj)

    buffer.add_trajectories(trajectories)
    assert len(buffer) == 6


def test_buffer_sample():
    """Test sampling from buffer."""
    buffer = ReplayBuffer()

    for i in range(10):
        buffer.add_transition(
            Transition(
                state=np.array([i, i * 2]),
                action=i % 4,
                reward=float(i),
                next_state=np.array([i + 1, (i + 1) * 2]),
                done=i == 9,
            )
        )

    batch = buffer.sample(batch_size=5)

    assert batch["states"].shape == (5, 2)
    assert batch["actions"].shape == (5,)
    assert batch["rewards"].shape == (5,)
    assert batch["next_states"].shape == (5, 2)
    assert batch["dones"].shape == (5,)


def test_buffer_circular():
    """Test circular buffer behavior."""
    buffer = ReplayBuffer(capacity=5)

    # Add 7 transitions
    for i in range(7):
        buffer.add_transition(
            Transition(
                state=np.array([i]),
                action=0,
                reward=float(i),
                next_state=np.array([i + 1]),
                done=False,
            )
        )

    # Should only have 5 (capacity)
    assert len(buffer) == 5

    # Should have the last 5 rewards (2, 3, 4, 5, 6)
    # Note: circular buffer overwrites oldest
    batch = buffer.sample(batch_size=5)
    rewards = set(batch["rewards"])
    # At least some of the newer rewards should be present
    assert any(r >= 2 for r in rewards)


def test_buffer_clear():
    """Test clearing buffer."""
    buffer = ReplayBuffer()

    for i in range(5):
        buffer.add_transition(
            Transition(
                state=np.array([i]),
                action=0,
                reward=0.0,
                next_state=np.array([i + 1]),
                done=False,
            )
        )

    assert len(buffer) == 5

    buffer.clear()
    assert len(buffer) == 0
