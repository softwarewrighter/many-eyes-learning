"""Tests for aggregation strategies."""

import numpy as np

from many_eyes.aggregation.simple import SimpleAggregator
from many_eyes.core import Trajectory, Transition


def make_trajectory(n_steps: int, reward: float = 0.0, final_reward: float = 0.0) -> Trajectory:
    """Helper to create a trajectory."""
    traj = Trajectory()
    for i in range(n_steps):
        is_last = i == n_steps - 1
        traj.add(
            Transition(
                state=np.array([i]),
                action=i % 4,
                reward=final_reward if is_last else reward,
                next_state=np.array([i + 1]),
                done=is_last,
            )
        )
    return traj


def test_simple_aggregator_empty():
    """Test aggregating empty list."""
    aggregator = SimpleAggregator()
    result = aggregator.aggregate([])

    assert result == []


def test_simple_aggregator_passthrough():
    """Test that simple aggregator passes through unchanged."""
    aggregator = SimpleAggregator()

    trajectories = [make_trajectory(5), make_trajectory(3), make_trajectory(7)]
    result = aggregator.aggregate(trajectories)

    assert len(result) == 3
    assert result is trajectories  # Same list


def test_simple_aggregator_stats_empty():
    """Test stats on empty trajectories."""
    aggregator = SimpleAggregator()
    stats = aggregator.stats([])

    assert stats["n_trajectories"] == 0
    assert stats["n_transitions"] == 0
    assert stats["total_reward"] == 0.0
    assert stats["success_rate"] == 0.0


def test_simple_aggregator_stats():
    """Test stats calculation."""
    aggregator = SimpleAggregator()

    trajectories = [
        make_trajectory(5, final_reward=1.0),  # Success
        make_trajectory(3, final_reward=0.0),  # Failure
        make_trajectory(4, final_reward=1.0),  # Success
    ]

    stats = aggregator.stats(trajectories)

    assert stats["n_trajectories"] == 3
    assert stats["n_transitions"] == 5 + 3 + 4
    assert stats["total_reward"] == 2.0
    assert abs(stats["success_rate"] - 2 / 3) < 1e-6


def test_simple_aggregator_stats_all_success():
    """Test stats when all trajectories succeed."""
    aggregator = SimpleAggregator()

    trajectories = [
        make_trajectory(5, final_reward=1.0),
        make_trajectory(3, final_reward=1.0),
    ]

    stats = aggregator.stats(trajectories)

    assert stats["success_rate"] == 1.0


def test_simple_aggregator_stats_all_failure():
    """Test stats when all trajectories fail."""
    aggregator = SimpleAggregator()

    trajectories = [
        make_trajectory(5, final_reward=0.0),
        make_trajectory(3, final_reward=0.0),
    ]

    stats = aggregator.stats(trajectories)

    assert stats["success_rate"] == 0.0
