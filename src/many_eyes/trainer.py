"""Training loop for Many-Eyes Learning."""

from dataclasses import dataclass, field

from many_eyes.aggregation.simple import SimpleAggregator
from many_eyes.core import Trajectory
from many_eyes.protocols import Aggregator, Environment, Learner, Scout


@dataclass
class TrainingConfig:
    """Configuration for training."""

    episodes: int = 100
    steps_per_episode: int = 100
    learning_steps: int = 10
    update_scouts_every: int = 10
    verbose: bool = True


@dataclass
class Trainer:
    """
    Coordinates scouts, aggregation, and learning.

    This is the main training loop that:
    1. Has scouts explore the environment
    2. Aggregates their experiences
    3. Updates the shared learner
    4. Optionally updates scout policies
    """

    env: Environment
    scouts: list[Scout]
    learner: Learner
    aggregator: Aggregator = field(default_factory=SimpleAggregator)
    config: TrainingConfig = field(default_factory=TrainingConfig)

    # Tracking
    _episode_rewards: list[float] = field(default_factory=list, init=False)
    _episode_successes: list[bool] = field(default_factory=list, init=False)

    def train(self) -> dict[str, list]:
        """
        Run the training loop.

        Returns:
            Dictionary with training history
        """
        history = {
            "episode_rewards": [],
            "success_rates": [],
            "losses": [],
        }

        for episode in range(self.config.episodes):
            # Exploration phase - each scout explores
            all_trajectories: list[Trajectory] = []
            for scout in self.scouts:
                env_copy = self.env.copy()
                trajectories = scout.explore(env_copy, self.config.steps_per_episode)
                all_trajectories.extend(trajectories)

            # Aggregation phase
            aggregated = self.aggregator.aggregate(all_trajectories)

            # Learning phase
            metrics = self.learner.update(aggregated, self.config.learning_steps)

            # Track metrics
            episode_reward = sum(t.total_reward for t in all_trajectories)
            successes = sum(1 for t in all_trajectories if t.reached_goal)
            success_rate = successes / len(all_trajectories) if all_trajectories else 0

            history["episode_rewards"].append(episode_reward)
            history["success_rates"].append(success_rate)
            history["losses"].append(metrics.get("loss", 0))

            # Update scout policies periodically
            if (episode + 1) % self.config.update_scouts_every == 0:
                policy = self.learner.get_policy()
                for scout in self.scouts:
                    scout.set_policy(policy)

            if self.config.verbose and (episode + 1) % 10 == 0:
                print(
                    f"Episode {episode + 1}/{self.config.episodes} | "
                    f"Reward: {episode_reward:.1f} | "
                    f"Success: {success_rate:.1%} | "
                    f"Loss: {metrics.get('loss', 0):.4f}"
                )

        return history


def train_single_scout(
    env: Environment,
    learner: Learner,
    config: TrainingConfig | None = None,
    seed: int | None = None,
) -> dict[str, list]:
    """
    Train with a single random scout (baseline).

    Args:
        env: Environment to train on
        learner: Learner to use
        config: Training configuration
        seed: Random seed

    Returns:
        Training history
    """
    from many_eyes.scouts.base import RandomScout

    config = config or TrainingConfig()
    scout = RandomScout(scout_id="single", seed=seed)

    trainer = Trainer(
        env=env,
        scouts=[scout],
        learner=learner,
        config=config,
    )

    return trainer.train()


def train_many_eyes(
    env: Environment,
    learner: Learner,
    n_scouts: int = 5,
    config: TrainingConfig | None = None,
    seed: int | None = None,
) -> dict[str, list]:
    """
    Train with multiple scouts (many-eyes approach).

    Args:
        env: Environment to train on
        learner: Learner to use
        n_scouts: Number of scouts
        config: Training configuration
        seed: Random seed

    Returns:
        Training history
    """
    from many_eyes.scouts.base import RandomScout
    from many_eyes.scouts.epsilon import EpsilonGreedyScout

    config = config or TrainingConfig()

    # Create diverse scouts
    scouts: list[Scout] = []
    for i in range(n_scouts):
        scout_seed = seed + i if seed is not None else None
        if i == 0:
            # One random scout
            scouts.append(RandomScout(scout_id=f"random_{i}", seed=scout_seed))
        else:
            # Rest are epsilon-greedy with varying epsilon
            epsilon = 0.1 + 0.2 * (i / n_scouts)
            scouts.append(
                EpsilonGreedyScout(
                    epsilon=epsilon,
                    scout_id=f"eps_{epsilon:.2f}_{i}",
                    seed=scout_seed,
                )
            )

    trainer = Trainer(
        env=env,
        scouts=scouts,
        learner=learner,
        config=config,
    )

    return trainer.train()
