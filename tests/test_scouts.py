"""Tests for scouts."""

import numpy as np

from many_eyes.envs.sparse_grid import SparseGridWorld
from many_eyes.scouts.base import RandomScout
from many_eyes.scouts.epsilon import EpsilonGreedyScout


def test_random_scout_creation():
    """Test creating a random scout."""
    scout = RandomScout(scout_id="test", seed=42)

    assert scout.name == "test"


def test_random_scout_explore():
    """Test random scout exploration."""
    env = SparseGridWorld(size=3, max_steps=10)
    scout = RandomScout(seed=42)

    trajectories = scout.explore(env, steps=20)

    assert len(trajectories) > 0
    total_transitions = sum(len(t) for t in trajectories)
    assert total_transitions <= 20


def test_random_scout_metadata():
    """Test trajectory metadata from random scout."""
    env = SparseGridWorld(size=3, max_steps=5)
    scout = RandomScout(scout_id="my_scout")

    trajectories = scout.explore(env, steps=10)

    assert trajectories[0].metadata["scout"] == "my_scout"
    assert trajectories[0].metadata["strategy"] == "random"


def test_epsilon_greedy_creation():
    """Test creating epsilon-greedy scout."""
    scout = EpsilonGreedyScout(epsilon=0.2, scout_id="test", seed=42)

    assert "epsilon" in scout.name or "test" in scout.name
    assert scout.epsilon == 0.2


def test_epsilon_greedy_explore_no_policy():
    """Test epsilon-greedy exploration without policy (falls back to random)."""
    env = SparseGridWorld(size=3, max_steps=10)
    scout = EpsilonGreedyScout(epsilon=0.5, seed=42)

    trajectories = scout.explore(env, steps=20)

    assert len(trajectories) > 0


def test_epsilon_greedy_with_policy():
    """Test epsilon-greedy with a policy."""
    env = SparseGridWorld(size=3, max_steps=10)
    scout = EpsilonGreedyScout(epsilon=0.0, seed=42)  # Always follow policy

    # Policy that always goes right
    def always_right(state: np.ndarray) -> int:
        return 1

    scout.set_policy(always_right)
    trajectories = scout.explore(env, steps=5)

    # With epsilon=0, should always follow policy
    for traj in trajectories:
        for t in traj:
            assert t.action == 1


def test_epsilon_greedy_explores():
    """Test that epsilon-greedy actually explores (takes random actions)."""
    env = SparseGridWorld(size=3, max_steps=50)
    scout = EpsilonGreedyScout(epsilon=1.0, seed=42)  # Always random

    # Policy that always goes right
    def always_right(state: np.ndarray) -> int:
        return 1

    scout.set_policy(always_right)
    trajectories = scout.explore(env, steps=50)

    # With epsilon=1.0, should see other actions too
    all_actions = [t.action for traj in trajectories for t in traj]
    unique_actions = set(all_actions)

    assert len(unique_actions) > 1  # Should have variety


def test_scouts_are_independent():
    """Test that different scouts produce different trajectories."""
    env = SparseGridWorld(size=5, max_steps=20)

    scout1 = RandomScout(seed=1)
    scout2 = RandomScout(seed=2)

    traj1 = scout1.explore(env.copy(), steps=20)
    traj2 = scout2.explore(env.copy(), steps=20)

    # Get action sequences
    actions1 = [t.action for traj in traj1 for t in traj]
    actions2 = [t.action for traj in traj2 for t in traj]

    # Different seeds should produce different sequences
    assert actions1 != actions2


def test_scout_respects_step_limit():
    """Test that scouts respect the step limit."""
    env = SparseGridWorld(size=5, max_steps=1000)  # High max_steps
    scout = RandomScout(seed=42)

    step_limit = 25
    trajectories = scout.explore(env, steps=step_limit)

    total = sum(len(t) for t in trajectories)
    assert total <= step_limit
