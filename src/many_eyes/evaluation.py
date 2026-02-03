"""Evaluation utilities for measuring learned policy performance."""

from dataclasses import dataclass

import numpy as np

from many_eyes.core import Trajectory, Transition
from many_eyes.protocols import Environment, Policy


@dataclass
class EvaluationResult:
    """Results from evaluating a policy."""

    n_episodes: int
    success_rate: float
    mean_reward: float
    mean_steps: float
    std_reward: float
    std_steps: float
    trajectories: list[Trajectory]

    def to_dict(self) -> dict:
        """Convert to dictionary for serialization."""
        return {
            "n_episodes": self.n_episodes,
            "success_rate": self.success_rate,
            "mean_reward": self.mean_reward,
            "mean_steps": self.mean_steps,
            "std_reward": self.std_reward,
            "std_steps": self.std_steps,
        }


def evaluate_policy(
    policy: Policy,
    env: Environment,
    n_episodes: int = 100,
    max_steps: int = 100,
) -> EvaluationResult:
    """
    Evaluate a learned policy with greedy action selection.

    This runs the policy WITHOUT exploration noise to measure
    what the agent has actually learned.

    Args:
        policy: The policy to evaluate
        env: Environment to evaluate in
        n_episodes: Number of evaluation episodes
        max_steps: Maximum steps per episode

    Returns:
        EvaluationResult with statistics
    """
    trajectories = []
    rewards = []
    steps_list = []
    successes = 0

    for _ in range(n_episodes):
        trajectory = Trajectory(metadata={"evaluation": True})
        state = env.reset()
        episode_reward = 0.0
        steps = 0

        for step in range(max_steps):
            action = policy(state)
            next_state, reward, done, info = env.step(action)

            trajectory.add(
                Transition(
                    state=state,
                    action=action,
                    reward=reward,
                    next_state=next_state,
                    done=done,
                )
            )

            episode_reward += reward
            steps = step + 1
            state = next_state

            if done:
                break

        trajectories.append(trajectory)
        rewards.append(episode_reward)
        steps_list.append(steps)

        if trajectory.reached_goal:
            successes += 1

    return EvaluationResult(
        n_episodes=n_episodes,
        success_rate=successes / n_episodes,
        mean_reward=float(np.mean(rewards)),
        mean_steps=float(np.mean(steps_list)),
        std_reward=float(np.std(rewards)),
        std_steps=float(np.std(steps_list)),
        trajectories=trajectories,
    )


def evaluate_random_baseline(
    env: Environment,
    n_episodes: int = 100,
    max_steps: int = 100,
    seed: int | None = None,
) -> EvaluationResult:
    """
    Evaluate random action selection as a baseline.

    Args:
        env: Environment to evaluate in
        n_episodes: Number of evaluation episodes
        max_steps: Maximum steps per episode
        seed: Random seed

    Returns:
        EvaluationResult with statistics
    """
    rng = np.random.default_rng(seed)

    def random_policy(state: np.ndarray) -> int:
        return int(rng.integers(0, env.n_actions))

    return evaluate_policy(random_policy, env, n_episodes, max_steps)
