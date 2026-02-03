"""Tests for core data structures."""

import numpy as np

from many_eyes.core import Trajectory, Transition


def test_transition_creation():
    """Test creating a transition."""
    t = Transition(
        state=np.array([1, 0, 0]),
        action=1,
        reward=0.5,
        next_state=np.array([0, 1, 0]),
        done=False,
    )

    assert t.action == 1
    assert t.reward == 0.5
    assert not t.done
    assert isinstance(t.state, np.ndarray)


def test_transition_converts_lists():
    """Test that lists are converted to arrays."""
    t = Transition(
        state=[1, 0, 0],
        action=0,
        reward=0.0,
        next_state=[0, 1, 0],
        done=False,
    )

    assert isinstance(t.state, np.ndarray)
    assert isinstance(t.next_state, np.ndarray)


def test_trajectory_empty():
    """Test empty trajectory."""
    traj = Trajectory()

    assert len(traj) == 0
    assert traj.total_reward == 0.0
    assert traj.final_state is None
    assert not traj.reached_goal


def test_trajectory_add():
    """Test adding transitions to trajectory."""
    traj = Trajectory()

    traj.add(
        Transition(
            state=np.array([1, 0]),
            action=0,
            reward=0.0,
            next_state=np.array([0, 1]),
            done=False,
        )
    )

    assert len(traj) == 1
    assert traj.total_reward == 0.0


def test_trajectory_total_reward():
    """Test total reward calculation."""
    traj = Trajectory()

    for i in range(5):
        traj.add(
            Transition(
                state=np.array([i]),
                action=0,
                reward=0.1 * i,
                next_state=np.array([i + 1]),
                done=i == 4,
            )
        )

    expected = 0.0 + 0.1 + 0.2 + 0.3 + 0.4
    assert abs(traj.total_reward - expected) < 1e-6


def test_trajectory_reached_goal():
    """Test reached_goal property."""
    # Trajectory with positive final reward
    traj_success = Trajectory()
    traj_success.add(
        Transition(
            state=np.array([0]),
            action=0,
            reward=1.0,
            next_state=np.array([1]),
            done=True,
        )
    )
    assert traj_success.reached_goal

    # Trajectory with zero final reward
    traj_fail = Trajectory()
    traj_fail.add(
        Transition(
            state=np.array([0]),
            action=0,
            reward=0.0,
            next_state=np.array([1]),
            done=True,
        )
    )
    assert not traj_fail.reached_goal


def test_trajectory_metadata():
    """Test trajectory metadata."""
    traj = Trajectory(metadata={"scout": "test", "episode": 1})

    assert traj.metadata["scout"] == "test"
    assert traj.metadata["episode"] == 1


def test_trajectory_iteration():
    """Test iterating over trajectory."""
    traj = Trajectory()
    for i in range(3):
        traj.add(
            Transition(
                state=np.array([i]),
                action=i,
                reward=0.0,
                next_state=np.array([i + 1]),
                done=False,
            )
        )

    actions = [t.action for t in traj]
    assert actions == [0, 1, 2]
