#!/usr/bin/env python3
"""
Diversity experiment: Compare homogeneous vs diverse scout configurations.

This experiment tests whether diversity of exploration strategies matters,
not just the number of scouts.

Configurations tested:
1. Homogeneous: 5 random scouts (same strategy)
2. Homogeneous: 5 epsilon-greedy scouts (same epsilon)
3. Diverse: Mix of random, epsilon-greedy, curious, optimistic
"""

import argparse
import json
from dataclasses import asdict, dataclass
from datetime import datetime
from pathlib import Path

import numpy as np

from many_eyes.envs.sparse_grid import SparseGridWorld
from many_eyes.evaluation import evaluate_policy, evaluate_random_baseline
from many_eyes.learner.dqn import DQNLearner
from many_eyes.scouts.base import RandomScout
from many_eyes.scouts.curious import CuriousScout
from many_eyes.scouts.epsilon import EpsilonGreedyScout
from many_eyes.scouts.optimistic import OptimisticScout
from many_eyes.visualization import Colors, print_training_progress


@dataclass
class DiversityConfig:
    """Configuration for diversity experiment."""

    grid_size: int = 7
    max_steps: int = 50
    training_episodes: int = 100
    steps_per_episode: int = 100
    learning_steps: int = 20
    eval_episodes: int = 100
    seeds: list[int] | None = None

    def __post_init__(self):
        if self.seeds is None:
            self.seeds = [42, 123, 456, 789, 1000]


def create_homogeneous_random(n_scouts: int, seed: int) -> list:
    """Create n identical random scouts."""
    return [RandomScout(scout_id=f"random_{i}", seed=seed + i) for i in range(n_scouts)]


def create_homogeneous_epsilon(n_scouts: int, seed: int, epsilon: float = 0.2) -> list:
    """Create n identical epsilon-greedy scouts."""
    return [
        EpsilonGreedyScout(epsilon=epsilon, scout_id=f"eps_{i}", seed=seed + i)
        for i in range(n_scouts)
    ]


def create_diverse_scouts(seed: int) -> list:
    """Create a diverse mix of scout strategies."""
    return [
        RandomScout(scout_id="random", seed=seed),
        EpsilonGreedyScout(epsilon=0.1, scout_id="eps_low", seed=seed + 1),
        EpsilonGreedyScout(epsilon=0.3, scout_id="eps_high", seed=seed + 2),
        CuriousScout(bonus_scale=0.5, scout_id="curious", seed=seed + 3),
        OptimisticScout(optimism=1.0, scout_id="optimistic", seed=seed + 4),
    ]


def run_configuration(
    config_name: str,
    scouts: list,
    env: SparseGridWorld,
    exp_config: DiversityConfig,
    seed: int,
    verbose: bool = True,
) -> dict:
    """Run training with a specific scout configuration."""
    n_scouts = len(scouts)

    # Create learner
    learner = DQNLearner(
        state_dim=env.size * env.size,
        n_actions=env.n_actions,
        hidden_dim=64,
        learning_rate=1e-3,
        gamma=0.95,
        batch_size=32,
        seed=seed,
    )

    steps_per_scout = exp_config.steps_per_episode // n_scouts
    history = {"episode_rewards": [], "success_rates": [], "losses": []}

    for episode in range(exp_config.training_episodes):
        all_trajectories = []
        for scout in scouts:
            env_copy = env.copy()
            trajectories = scout.explore(env_copy, steps_per_scout)
            all_trajectories.extend(trajectories)

        metrics = learner.update(all_trajectories, exp_config.learning_steps)

        episode_reward = sum(t.total_reward for t in all_trajectories)
        successes = sum(1 for t in all_trajectories if t.reached_goal)
        success_rate = successes / len(all_trajectories) if all_trajectories else 0

        history["episode_rewards"].append(episode_reward)
        history["success_rates"].append(success_rate)
        history["losses"].append(metrics.get("loss", 0))

        if (episode + 1) % 5 == 0:
            policy = learner.get_policy()
            for scout in scouts:
                scout.set_policy(policy)

        if verbose:
            print_training_progress(
                episode + 1,
                exp_config.training_episodes,
                episode_reward,
                success_rate,
                metrics.get("loss", 0),
                config_name,
            )

    if verbose:
        print()

    # Evaluate
    policy = learner.get_policy()
    eval_result = evaluate_policy(
        policy, env.copy(), n_episodes=exp_config.eval_episodes, max_steps=exp_config.max_steps
    )

    return {
        "config_name": config_name,
        "n_scouts": n_scouts,
        "seed": seed,
        "training_history": history,
        "eval_result": eval_result.to_dict(),
    }


def run_diversity_experiment(exp_config: DiversityConfig, output_dir: Path, verbose: bool = True):
    """Run the full diversity experiment."""
    results = []
    output_dir.mkdir(parents=True, exist_ok=True)

    env = SparseGridWorld(size=exp_config.grid_size, max_steps=exp_config.max_steps)

    # Random baseline
    if verbose:
        print(f"\n{Colors.BOLD}Evaluating random baseline...{Colors.RESET}")
    random_eval = evaluate_random_baseline(env, n_episodes=exp_config.eval_episodes)
    results.append(
        {
            "config_name": "random_baseline",
            "n_scouts": 0,
            "seed": 0,
            "training_history": {},
            "eval_result": random_eval.to_dict(),
        }
    )

    configurations = [
        ("homogeneous_random", create_homogeneous_random),
        ("homogeneous_epsilon", lambda n, s: create_homogeneous_epsilon(n, s, 0.2)),
        ("diverse", lambda n, s: create_diverse_scouts(s)),
    ]

    total_runs = len(exp_config.seeds) * len(configurations)
    run_idx = 0

    for seed in exp_config.seeds:
        for config_name, create_fn in configurations:
            run_idx += 1
            if verbose:
                status = f"[{run_idx}/{total_runs}] {config_name}, seed={seed}"
                print(f"\n{Colors.BOLD}{status}{Colors.RESET}")

            scouts = create_fn(5, seed)  # Always 5 scouts for fair comparison
            result = run_configuration(config_name, scouts, env, exp_config, seed, verbose)
            results.append(result)

    # Save results
    output_file = output_dir / "diversity_results.json"
    experiment_data = {
        "config": asdict(exp_config),
        "timestamp": datetime.now().isoformat(),
        "experiment_type": "diversity",
        "results": results,
    }

    with open(output_file, "w") as f:
        json.dump(experiment_data, f, indent=2)

    if verbose:
        print(f"\n{Colors.GREEN}Results saved to: {output_file}{Colors.RESET}")

    return experiment_data


def summarize_diversity_results(data: dict) -> dict[str, dict]:
    """Summarize results by configuration."""
    results_by_config = {}

    for result in data["results"]:
        config = result["config_name"]
        if config not in results_by_config:
            results_by_config[config] = []
        results_by_config[config].append(result)

    summary = {}
    for config, runs in results_by_config.items():
        if config == "random_baseline":
            summary[config] = runs[0]["eval_result"]
        else:
            success_rates = [r["eval_result"]["success_rate"] for r in runs]
            mean_rewards = [r["eval_result"]["mean_reward"] for r in runs]

            summary[config] = {
                "n_runs": len(runs),
                "success_rate": float(np.mean(success_rates)),
                "success_rate_std": float(np.std(success_rates)),
                "mean_reward": float(np.mean(mean_rewards)),
                "mean_reward_std": float(np.std(mean_rewards)),
            }

    return summary


def main():
    parser = argparse.ArgumentParser(description="Run diversity experiment")
    parser.add_argument("--output-dir", default="experiments/results")
    parser.add_argument("--episodes", type=int, default=100)
    parser.add_argument("--seeds", type=int, nargs="+", default=[42, 123, 456, 789, 1000])
    parser.add_argument("--grid-size", type=int, default=7)
    parser.add_argument("--quiet", action="store_true")

    args = parser.parse_args()

    config = DiversityConfig(
        grid_size=args.grid_size,
        training_episodes=args.episodes,
        seeds=args.seeds,
    )

    print(f"{Colors.BOLD}Diversity Experiment{Colors.RESET}")
    print(f"Grid: {config.grid_size}x{config.grid_size}")
    print(f"Episodes: {config.training_episodes}")
    print(f"Seeds: {config.seeds}")
    print("\nConfigurations:")
    print("  - homogeneous_random: 5 random scouts")
    print("  - homogeneous_epsilon: 5 epsilon-greedy (eps=0.2)")
    print("  - diverse: random + 2 epsilon + curious + optimistic")

    data = run_diversity_experiment(config, Path(args.output_dir), not args.quiet)

    # Print summary
    print(f"\n{Colors.BOLD}{'=' * 60}{Colors.RESET}")
    print(f"{Colors.BOLD}DIVERSITY EXPERIMENT SUMMARY{Colors.RESET}")
    print(f"{Colors.BOLD}{'=' * 60}{Colors.RESET}")

    summary = summarize_diversity_results(data)

    print(f"\n{'Configuration':<25} {'Success Rate':>20} {'Mean Reward':>15}")
    print("-" * 62)

    for config_name in ["random_baseline", "homogeneous_random", "homogeneous_epsilon", "diverse"]:
        if config_name in summary:
            stats = summary[config_name]
            if "success_rate_std" in stats:
                sr = f"{stats['success_rate']:.1%} +/- {stats['success_rate_std']:.1%}"
                mr = f"{stats['mean_reward']:.2f}"
            else:
                sr = f"{stats['success_rate']:.1%}"
                mr = f"{stats['mean_reward']:.2f}"
            print(f"{config_name:<25} {sr:>20} {mr:>15}")

    print("-" * 62)

    # Highlight key finding
    if "diverse" in summary and "homogeneous_random" in summary:
        div_sr = summary["diverse"]["success_rate"]
        hom_sr = summary["homogeneous_random"]["success_rate"]
        if div_sr > hom_sr:
            improvement = (div_sr - hom_sr) / max(hom_sr, 0.01) * 100
            print(
                f"\n{Colors.GREEN}Diverse scouts outperform homogeneous by "
                f"{improvement:.0f}%{Colors.RESET}"
            )


if __name__ == "__main__":
    main()
