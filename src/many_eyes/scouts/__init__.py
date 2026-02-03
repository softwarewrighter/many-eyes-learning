"""Scout implementations."""

from many_eyes.scouts.base import RandomScout
from many_eyes.scouts.epsilon import EpsilonGreedyScout

__all__ = ["RandomScout", "EpsilonGreedyScout"]
