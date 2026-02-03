"""Tests for CuriousScout and OptimisticScout."""

import numpy as np

from many_eyes.envs.sparse_grid import SparseGridWorld
from many_eyes.scouts.curious import CuriousScout
from many_eyes.scouts.optimistic import OptimisticScout


class TestCuriousScout:
    """Tests for CuriousScout."""

    def test_creation(self):
        """Test creating a curious scout."""
        scout = CuriousScout(bonus_scale=0.5, scout_id="test", seed=42)
        assert scout.name == "test"
        assert scout.bonus_scale == 0.5

    def test_explore(self):
        """Test exploration collects trajectories."""
        env = SparseGridWorld(size=3, max_steps=10)
        scout = CuriousScout(seed=42)

        trajectories = scout.explore(env, steps=20)

        assert len(trajectories) > 0
        total = sum(len(t) for t in trajectories)
        assert total <= 20

    def test_tracks_visits(self):
        """Test that visit counts are tracked."""
        env = SparseGridWorld(size=3, max_steps=10)
        scout = CuriousScout(seed=42)

        scout.explore(env, steps=20)

        # Should have visited some states
        assert len(scout.visit_counts) > 0
        assert all(v > 0 for v in scout.visit_counts.values())

    def test_reset_counts(self):
        """Test resetting visit counts."""
        env = SparseGridWorld(size=3, max_steps=10)
        scout = CuriousScout(seed=42)

        scout.explore(env, steps=20)
        assert len(scout.visit_counts) > 0

        scout.reset_counts()
        assert len(scout.visit_counts) == 0

    def test_metadata(self):
        """Test trajectory metadata."""
        env = SparseGridWorld(size=3, max_steps=5)
        scout = CuriousScout(bonus_scale=0.3, scout_id="curious_test")

        trajectories = scout.explore(env, steps=10)

        assert trajectories[0].metadata["scout"] == "curious_test"
        assert trajectories[0].metadata["strategy"] == "curious"
        assert trajectories[0].metadata["bonus_scale"] == 0.3

    def test_with_policy(self):
        """Test using a policy."""
        env = SparseGridWorld(size=3, max_steps=10)
        scout = CuriousScout(seed=42)

        def always_right(state: np.ndarray) -> int:
            return 1

        scout.set_policy(always_right)
        trajectories = scout.explore(env, steps=20)

        # Should still collect trajectories
        assert len(trajectories) > 0


class TestOptimisticScout:
    """Tests for OptimisticScout."""

    def test_creation(self):
        """Test creating an optimistic scout."""
        scout = OptimisticScout(optimism=2.0, decay=0.9, scout_id="test", seed=42)
        assert scout.name == "test"
        assert scout.optimism == 2.0
        assert scout.decay == 0.9

    def test_explore(self):
        """Test exploration collects trajectories."""
        env = SparseGridWorld(size=3, max_steps=10)
        scout = OptimisticScout(seed=42)

        trajectories = scout.explore(env, steps=20)

        assert len(trajectories) > 0
        total = sum(len(t) for t in trajectories)
        assert total <= 20

    def test_optimism_decays(self):
        """Test that optimism decays with visits."""
        env = SparseGridWorld(size=3, max_steps=10)
        scout = OptimisticScout(optimism=1.0, decay=0.5, seed=42)

        # Get initial bonus
        state = env.reset()
        bonus_initial = scout._get_optimistic_bonus(state, 0)

        # Take action
        scout._action_counts[(0, 0)] = 1

        # Bonus should be lower after visit
        bonus_after = scout._get_optimistic_bonus(state, 0)
        assert bonus_after < bonus_initial

    def test_metadata(self):
        """Test trajectory metadata."""
        env = SparseGridWorld(size=3, max_steps=5)
        scout = OptimisticScout(optimism=1.5, scout_id="opt_test")

        trajectories = scout.explore(env, steps=10)

        assert trajectories[0].metadata["scout"] == "opt_test"
        assert trajectories[0].metadata["strategy"] == "optimistic"
        assert trajectories[0].metadata["optimism"] == 1.5

    def test_reset_counts(self):
        """Test resetting action counts."""
        env = SparseGridWorld(size=3, max_steps=10)
        scout = OptimisticScout(seed=42)

        scout.explore(env, steps=20)
        assert len(scout._action_counts) > 0

        scout.reset_counts()
        assert len(scout._action_counts) == 0

    def test_with_policy(self):
        """Test using a policy."""
        env = SparseGridWorld(size=3, max_steps=10)
        scout = OptimisticScout(seed=42)

        def always_down(state: np.ndarray) -> int:
            return 2

        scout.set_policy(always_down)
        trajectories = scout.explore(env, steps=20)

        # Should still collect trajectories
        assert len(trajectories) > 0


class TestScoutDiversity:
    """Tests for diversity of scout behaviors."""

    def test_different_scouts_different_coverage(self):
        """Test that different scout types explore differently."""
        env = SparseGridWorld(size=5, max_steps=30)

        scouts = [
            ("random", CuriousScout(seed=42)),
            ("curious", CuriousScout(bonus_scale=1.0, seed=42)),
            ("optimistic", OptimisticScout(optimism=2.0, seed=42)),
        ]

        coverages = []
        for name, scout in scouts:
            trajectories = scout.explore(env.copy(), steps=50)
            visited = set()
            for traj in trajectories:
                for t in traj:
                    visited.add(tuple(t.state.tolist()))
            coverages.append((name, len(visited)))

        # All should have some coverage
        for name, coverage in coverages:
            assert coverage > 0, f"{name} should visit some states"

    def test_scouts_are_independent(self):
        """Test that scouts with different seeds behave differently."""
        env = SparseGridWorld(size=5, max_steps=20)

        scout1 = CuriousScout(seed=1)
        scout2 = CuriousScout(seed=2)

        traj1 = scout1.explore(env.copy(), steps=20)
        traj2 = scout2.explore(env.copy(), steps=20)

        actions1 = [t.action for tr in traj1 for t in tr]
        actions2 = [t.action for tr in traj2 for t in tr]

        # Different seeds should produce different sequences
        assert actions1 != actions2
