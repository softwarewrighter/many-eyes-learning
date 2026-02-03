#!/usr/bin/env python3
"""
Demo: Single Scout vs Many Eyes

This script demonstrates the core idea of many-eyes learning:
multiple scouts exploring a sparse-reward environment lead to
better learning outcomes than a single explorer.
"""

import matplotlib.pyplot as plt
import numpy as np

from many_eyes.envs.sparse_grid import SparseGridWorld
from many_eyes.learner.dqn import DQNLearner
from many_eyes.trainer import TrainingConfig, train_many_eyes, train_single_scout


def run_comparison(seed: int = 42) -> dict:
    """Run single vs many-eyes comparison."""
    print("=" * 60)
    print("Many-Eyes Learning Demo")
    print("=" * 60)
    print()

    # Environment: 5x5 sparse grid
    env = SparseGridWorld(size=5, max_steps=50)
    print(f"Environment: {env.size}x{env.size} grid")
    print(f"Goal: reach ({env.goal[0]}, {env.goal[1]}) from (0, 0)")
    print("Reward: +1 at goal, 0 elsewhere (sparse)")
    print()
    print("Initial grid:")
    print(env.render())
    print()

    config = TrainingConfig(
        episodes=50,
        steps_per_episode=100,
        learning_steps=10,
        update_scouts_every=5,
        verbose=True,
    )

    # Single scout baseline
    print("-" * 60)
    print("Training with SINGLE scout (baseline)")
    print("-" * 60)

    learner_single = DQNLearner(
        state_dim=env.size * env.size,
        n_actions=env.n_actions,
        seed=seed,
    )
    history_single = train_single_scout(env, learner_single, config, seed=seed)

    # Many-eyes approach
    print()
    print("-" * 60)
    print("Training with MANY EYES (5 scouts)")
    print("-" * 60)

    learner_many = DQNLearner(
        state_dim=env.size * env.size,
        n_actions=env.n_actions,
        seed=seed,
    )
    history_many = train_many_eyes(env, learner_many, n_scouts=5, config=config, seed=seed)

    return {
        "single": history_single,
        "many": history_many,
    }


def plot_results(results: dict, save_path: str | None = None):
    """Plot comparison of single vs many-eyes."""
    fig, axes = plt.subplots(1, 2, figsize=(12, 4))

    # Success rate
    ax = axes[0]
    episodes = range(1, len(results["single"]["success_rates"]) + 1)

    ax.plot(episodes, results["single"]["success_rates"], label="Single Scout", alpha=0.7)
    ax.plot(episodes, results["many"]["success_rates"], label="Many Eyes (5)", alpha=0.7)
    ax.set_xlabel("Episode")
    ax.set_ylabel("Success Rate")
    ax.set_title("Success Rate: Single vs Many Eyes")
    ax.legend()
    ax.grid(True, alpha=0.3)

    # Cumulative reward
    ax = axes[1]
    cum_single = np.cumsum(results["single"]["episode_rewards"])
    cum_many = np.cumsum(results["many"]["episode_rewards"])

    ax.plot(episodes, cum_single, label="Single Scout", alpha=0.7)
    ax.plot(episodes, cum_many, label="Many Eyes (5)", alpha=0.7)
    ax.set_xlabel("Episode")
    ax.set_ylabel("Cumulative Reward")
    ax.set_title("Cumulative Reward: Single vs Many Eyes")
    ax.legend()
    ax.grid(True, alpha=0.3)

    plt.tight_layout()

    if save_path:
        plt.savefig(save_path, dpi=150, bbox_inches="tight")
        print(f"\nPlot saved to: {save_path}")
    else:
        plt.show()


def print_summary(results: dict):
    """Print summary statistics."""
    print()
    print("=" * 60)
    print("Summary")
    print("=" * 60)

    for name, history in results.items():
        total_reward = sum(history["episode_rewards"])
        avg_success = np.mean(history["success_rates"])
        final_success = np.mean(history["success_rates"][-10:])

        print(f"\n{name.upper()}:")
        print(f"  Total reward:      {total_reward:.1f}")
        print(f"  Avg success rate:  {avg_success:.1%}")
        print(f"  Final success rate (last 10): {final_success:.1%}")


def main():
    """Run the demo."""
    results = run_comparison(seed=42)
    print_summary(results)

    # Try to plot (may fail if no display)
    try:
        plot_results(results, save_path="experiments/comparison.png")
    except Exception as e:
        print(f"\nCould not create plot: {e}")
        print("(This is fine if running headless)")


if __name__ == "__main__":
    main()
