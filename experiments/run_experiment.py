#!/usr/bin/env python3
"""
Run reproducible experiments comparing single vs many-eyes learning.

This script runs multiple seeds and saves all results to JSON for
later analysis and plotting.
"""

import argparse
import json
import time
from dataclasses import asdict, dataclass
from datetime import datetime
from pathlib import Path

import numpy as np

from many_eyes.envs.sparse_grid import SparseGridWorld
from many_eyes.evaluation import evaluate_policy, evaluate_random_baseline
from many_eyes.learner.dqn import DQNLearner
from many_eyes.scouts.base import RandomScout
from many_eyes.scouts.epsilon import EpsilonGreedyScout
from many_eyes.trainer import TrainingConfig
from many_eyes.visualization import Colors, print_training_progress


@dataclass
class ExperimentConfig:
    """Configuration for an experiment."""

    name: str
    grid_size: int = 5
    max_steps_per_episode: int = 50
    training_episodes: int = 100
    steps_per_episode: int = 100
    learning_steps: int = 20
    eval_episodes: int = 100
    seeds: list[int] | None = None
    n_scouts_list: list[int] | None = None

    def __post_init__(self):
        if self.seeds is None:
            self.seeds = [42, 123, 456, 789, 1000]
        if self.n_scouts_list is None:
            self.n_scouts_list = [1, 3, 5, 10]


@dataclass
class RunResult:
    """Results from a single training run."""

    method: str
    n_scouts: int
    seed: int
    training_history: dict
    eval_result: dict
    training_time: float


def create_scouts(n_scouts: int, seed: int) -> list:
    """Create a diverse set of scouts."""
    scouts = []
    for i in range(n_scouts):
        scout_seed = seed + i * 1000
        if n_scouts == 1:
            # Single scout: just random
            scouts.append(RandomScout(scout_id=f"single_{seed}", seed=scout_seed))
        else:
            # Many scouts: mix of strategies
            if i == 0:
                scouts.append(RandomScout(scout_id=f"random_{i}", seed=scout_seed))
            else:
                epsilon = 0.05 + 0.25 * (i / n_scouts)
                scouts.append(
                    EpsilonGreedyScout(
                        epsilon=epsilon,
                        scout_id=f"eps_{epsilon:.2f}_{i}",
                        seed=scout_seed,
                    )
                )
    return scouts


def run_single_experiment(
    env: SparseGridWorld,
    n_scouts: int,
    config: ExperimentConfig,
    seed: int,
    verbose: bool = True,
) -> RunResult:
    """Run a single training experiment."""
    method = "single" if n_scouts == 1 else f"many_eyes_{n_scouts}"

    # Create learner
    learner = DQNLearner(
        state_dim=env.size * env.size,
        n_actions=env.n_actions,
        hidden_dim=64,
        learning_rate=1e-3,
        gamma=0.95,  # Lower gamma for faster credit assignment with sparse rewards
        batch_size=32,
        seed=seed,
    )

    # Create scouts
    scouts = create_scouts(n_scouts, seed)

    # Training config - IMPORTANT: scale steps so total env interactions are fair
    # Single scout gets all steps, many scouts share steps
    steps_per_scout = config.steps_per_episode // n_scouts

    TrainingConfig(
        episodes=config.training_episodes,
        steps_per_episode=steps_per_scout,
        learning_steps=config.learning_steps,
        update_scouts_every=5,
        verbose=False,
    )

    # Train
    start_time = time.time()

    # Note: We run the training loop manually to track metrics
    # The Trainer class is available for simpler use cases
    history = {"episode_rewards": [], "success_rates": [], "losses": []}

    for episode in range(config.training_episodes):
        # Exploration
        all_trajectories = []
        for scout in scouts:
            env_copy = env.copy()
            trajectories = scout.explore(env_copy, steps_per_scout)
            all_trajectories.extend(trajectories)

        # Learn
        metrics = learner.update(all_trajectories, config.learning_steps)

        # Track
        episode_reward = sum(t.total_reward for t in all_trajectories)
        successes = sum(1 for t in all_trajectories if t.reached_goal)
        success_rate = successes / len(all_trajectories) if all_trajectories else 0

        history["episode_rewards"].append(episode_reward)
        history["success_rates"].append(success_rate)
        history["losses"].append(metrics.get("loss", 0))

        # Update scout policies
        if (episode + 1) % 5 == 0:
            policy = learner.get_policy()
            for scout in scouts:
                scout.set_policy(policy)

        if verbose:
            print_training_progress(
                episode + 1,
                config.training_episodes,
                episode_reward,
                success_rate,
                metrics.get("loss", 0),
                method,
            )

    training_time = time.time() - start_time

    if verbose:
        print()

    # Evaluate learned policy (greedy, no exploration)
    policy = learner.get_policy()
    eval_result = evaluate_policy(
        policy,
        env.copy(),
        n_episodes=config.eval_episodes,
        max_steps=config.max_steps_per_episode,
    )

    return RunResult(
        method=method,
        n_scouts=n_scouts,
        seed=seed,
        training_history=history,
        eval_result=eval_result.to_dict(),
        training_time=training_time,
    )


def run_full_experiment(config: ExperimentConfig, output_dir: Path, verbose: bool = True):
    """Run full experiment with multiple seeds and scout configurations."""
    results = []
    output_dir.mkdir(parents=True, exist_ok=True)

    env = SparseGridWorld(size=config.grid_size, max_steps=config.max_steps_per_episode)

    # Random baseline
    if verbose:
        print(f"\n{Colors.BOLD}Evaluating random baseline...{Colors.RESET}")
    random_eval = evaluate_random_baseline(env, n_episodes=config.eval_episodes)
    baseline_result = {
        "method": "random_baseline",
        "n_scouts": 0,
        "seed": 0,
        "training_history": {},
        "eval_result": random_eval.to_dict(),
        "training_time": 0,
    }
    results.append(baseline_result)

    # Run experiments
    total_runs = len(config.seeds) * len(config.n_scouts_list)
    run_idx = 0

    for n_scouts in config.n_scouts_list:
        for seed in config.seeds:
            run_idx += 1
            method = "single" if n_scouts == 1 else f"many_eyes_{n_scouts}"

            if verbose:
                status = f"[{run_idx}/{total_runs}] {method}, seed={seed}"
                print(f"\n{Colors.BOLD}{status}{Colors.RESET}")

            result = run_single_experiment(env, n_scouts, config, seed, verbose)
            results.append(asdict(result))

    # Save results
    output_file = output_dir / "results.json"
    experiment_data = {
        "config": asdict(config),
        "timestamp": datetime.now().isoformat(),
        "results": results,
    }

    with open(output_file, "w") as f:
        json.dump(experiment_data, f, indent=2)

    if verbose:
        print(f"\n{Colors.GREEN}Results saved to: {output_file}{Colors.RESET}")

    return experiment_data


def summarize_results(data: dict) -> dict[str, dict]:
    """Summarize results by method, aggregating across seeds."""
    results_by_method = {}

    for result in data["results"]:
        method = result["method"]
        if method not in results_by_method:
            results_by_method[method] = []
        results_by_method[method].append(result)

    summary = {}
    for method, runs in results_by_method.items():
        if method == "random_baseline":
            summary[method] = runs[0]["eval_result"]
        else:
            success_rates = [r["eval_result"]["success_rate"] for r in runs]
            mean_rewards = [r["eval_result"]["mean_reward"] for r in runs]
            training_times = [r["training_time"] for r in runs]

            summary[method] = {
                "n_runs": len(runs),
                "success_rate": float(np.mean(success_rates)),
                "success_rate_std": float(np.std(success_rates)),
                "mean_reward": float(np.mean(mean_rewards)),
                "mean_reward_std": float(np.std(mean_rewards)),
                "training_time": float(np.mean(training_times)),
            }

    return summary


def main():
    parser = argparse.ArgumentParser(description="Run many-eyes experiments")
    parser.add_argument("--output-dir", default="experiments/results")
    parser.add_argument("--episodes", type=int, default=100)
    parser.add_argument("--seeds", type=int, nargs="+", default=[42, 123, 456])
    parser.add_argument("--scouts", type=int, nargs="+", default=[1, 3, 5])
    parser.add_argument("--grid-size", type=int, default=5)
    parser.add_argument("--quiet", action="store_true")

    args = parser.parse_args()

    config = ExperimentConfig(
        name="single_vs_many_eyes",
        grid_size=args.grid_size,
        training_episodes=args.episodes,
        seeds=args.seeds,
        n_scouts_list=args.scouts,
    )

    print(f"{Colors.BOLD}Many-Eyes Learning Experiment{Colors.RESET}")
    print(f"Grid: {config.grid_size}x{config.grid_size}")
    print(f"Episodes: {config.training_episodes}")
    print(f"Seeds: {config.seeds}")
    print(f"Scout configurations: {config.n_scouts_list}")

    data = run_full_experiment(config, Path(args.output_dir), verbose=not args.quiet)

    # Print summary
    print(f"\n{Colors.BOLD}{'=' * 60}{Colors.RESET}")
    print(f"{Colors.BOLD}SUMMARY (Evaluation on Learned Policy){Colors.RESET}")
    print(f"{Colors.BOLD}{'=' * 60}{Colors.RESET}")

    summary = summarize_results(data)

    print(f"\n{'Method':<20} {'Success Rate':>15} {'Mean Reward':>15}")
    print("-" * 52)

    for method, stats in sorted(summary.items()):
        if "success_rate_std" in stats:
            sr = f"{stats['success_rate']:.1%} +/- {stats['success_rate_std']:.1%}"
            mr = f"{stats['mean_reward']:.2f} +/- {stats.get('mean_reward_std', 0):.2f}"
        else:
            sr = f"{stats['success_rate']:.1%}"
            mr = f"{stats['mean_reward']:.2f}"
        print(f"{method:<20} {sr:>15} {mr:>15}")

    print("-" * 52)


if __name__ == "__main__":
    main()
