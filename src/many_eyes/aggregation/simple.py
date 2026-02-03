"""Simple aggregation strategy."""

from dataclasses import dataclass

from many_eyes.core import Trajectory


@dataclass
class SimpleAggregator:
    """
    Simple aggregator that pools all experiences equally.

    No filtering or weighting - all trajectories are treated the same.
    """

    def aggregate(self, trajectories: list[Trajectory]) -> list[Trajectory]:
        """
        Return all trajectories unchanged.

        This is the simplest aggregation strategy - just pool everything.

        Args:
            trajectories: All trajectories from all scouts

        Returns:
            Same trajectories (no filtering)
        """
        return trajectories

    def stats(self, trajectories: list[Trajectory]) -> dict[str, float]:
        """Compute statistics about the trajectories."""
        if not trajectories:
            return {
                "n_trajectories": 0,
                "n_transitions": 0,
                "total_reward": 0.0,
                "success_rate": 0.0,
            }

        n_transitions = sum(len(t) for t in trajectories)
        total_reward = sum(t.total_reward for t in trajectories)
        successes = sum(1 for t in trajectories if t.reached_goal)

        return {
            "n_trajectories": len(trajectories),
            "n_transitions": n_transitions,
            "total_reward": total_reward,
            "success_rate": successes / len(trajectories),
        }
