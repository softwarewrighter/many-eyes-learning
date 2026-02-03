"""Scout implementations."""

from many_eyes.scouts.base import RandomScout
from many_eyes.scouts.curious import CuriousScout
from many_eyes.scouts.epsilon import EpsilonGreedyScout
from many_eyes.scouts.optimistic import OptimisticScout

__all__ = ["RandomScout", "EpsilonGreedyScout", "CuriousScout", "OptimisticScout"]
