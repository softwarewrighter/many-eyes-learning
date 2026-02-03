"""Many-Eyes Learning: Structured exploration for sparse rewards."""

from many_eyes.core import Trajectory, Transition
from many_eyes.protocols import Aggregator, Environment, Learner, Scout

__version__ = "0.1.0"

__all__ = [
    "Trajectory",
    "Transition",
    "Scout",
    "Environment",
    "Aggregator",
    "Learner",
]
