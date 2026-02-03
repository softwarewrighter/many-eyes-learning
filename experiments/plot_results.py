#!/usr/bin/env python3
"""
Generate plots from experiment results.

Reads results.json and creates publication-quality plots.
"""

import argparse
import json
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np


def load_results(results_file: Path) -> dict:
    """Load results from JSON file."""
    with open(results_file) as f:
        return json.load(f)


def plot_success_rate_comparison(data: dict, output_dir: Path):
    """Plot success rate comparison across methods."""
    results_by_method = {}

    for result in data["results"]:
        method = result["method"]
        if method not in results_by_method:
            results_by_method[method] = []
        results_by_method[method].append(result["eval_result"]["success_rate"])

    methods = []
    means = []
    stds = []

    for method in ["random_baseline", "single", "many_eyes_3", "many_eyes_5", "many_eyes_10"]:
        if method in results_by_method:
            methods.append(method.replace("_", "\n"))
            rates = results_by_method[method]
            means.append(np.mean(rates))
            stds.append(np.std(rates) if len(rates) > 1 else 0)

    fig, ax = plt.subplots(figsize=(10, 6))

    colors = ["gray", "steelblue", "forestgreen", "darkorange", "crimson"]
    bars = ax.bar(methods, means, yerr=stds, capsize=5, color=colors[: len(methods)], alpha=0.8)

    ax.set_ylabel("Success Rate", fontsize=12)
    ax.set_title("Learned Policy Performance: Single vs Many-Eyes", fontsize=14)
    ax.set_ylim(0, 1)
    ax.axhline(y=means[0], color="gray", linestyle="--", alpha=0.5, label="Random baseline")

    # Add value labels
    for bar, mean, std in zip(bars, means, stds):
        height = bar.get_height()
        label = f"{mean:.1%}"
        if std > 0:
            label += f"\n(+/-{std:.1%})"
        ax.annotate(
            label,
            xy=(bar.get_x() + bar.get_width() / 2, height),
            xytext=(0, 3),
            textcoords="offset points",
            ha="center",
            va="bottom",
            fontsize=9,
        )

    plt.tight_layout()
    output_file = output_dir / "success_rate_comparison.png"
    plt.savefig(output_file, dpi=150, bbox_inches="tight")
    print(f"Saved: {output_file}")
    plt.close()


def plot_learning_curves(data: dict, output_dir: Path):
    """Plot learning curves showing training progress."""
    fig, axes = plt.subplots(1, 2, figsize=(14, 5))

    # Group by method
    results_by_method = {}
    for result in data["results"]:
        method = result["method"]
        if method == "random_baseline":
            continue
        if method not in results_by_method:
            results_by_method[method] = []
        if result["training_history"]:
            results_by_method[method].append(result["training_history"])

    colors = {
        "single": "steelblue",
        "many_eyes_3": "forestgreen",
        "many_eyes_5": "darkorange",
        "many_eyes_10": "crimson",
    }

    # Success rate over episodes
    ax = axes[0]
    for method, histories in results_by_method.items():
        if not histories:
            continue
        # Average across seeds
        all_success = np.array([h["success_rates"] for h in histories])
        mean_success = np.mean(all_success, axis=0)
        std_success = np.std(all_success, axis=0)
        episodes = range(1, len(mean_success) + 1)

        color = colors.get(method, "gray")
        label = method.replace("_", " ")
        ax.plot(episodes, mean_success, label=label, color=color, linewidth=2)
        ax.fill_between(
            episodes, mean_success - std_success, mean_success + std_success, alpha=0.2, color=color
        )

    ax.set_xlabel("Episode", fontsize=12)
    ax.set_ylabel("Training Success Rate", fontsize=12)
    ax.set_title("Learning Progress During Training", fontsize=14)
    ax.legend()
    ax.grid(True, alpha=0.3)

    # Cumulative reward
    ax = axes[1]
    for method, histories in results_by_method.items():
        if not histories:
            continue
        all_rewards = np.array([np.cumsum(h["episode_rewards"]) for h in histories])
        mean_rewards = np.mean(all_rewards, axis=0)
        std_rewards = np.std(all_rewards, axis=0)
        episodes = range(1, len(mean_rewards) + 1)

        color = colors.get(method, "gray")
        label = method.replace("_", " ")
        ax.plot(episodes, mean_rewards, label=label, color=color, linewidth=2)
        ax.fill_between(
            episodes, mean_rewards - std_rewards, mean_rewards + std_rewards, alpha=0.2, color=color
        )

    ax.set_xlabel("Episode", fontsize=12)
    ax.set_ylabel("Cumulative Reward", fontsize=12)
    ax.set_title("Cumulative Reward During Training", fontsize=14)
    ax.legend()
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    output_file = output_dir / "learning_curves.png"
    plt.savefig(output_file, dpi=150, bbox_inches="tight")
    print(f"Saved: {output_file}")
    plt.close()


def plot_scouts_scaling(data: dict, output_dir: Path):
    """Plot how performance scales with number of scouts."""
    results_by_scouts = {}

    for result in data["results"]:
        if result["method"] == "random_baseline":
            continue
        n_scouts = result["n_scouts"]
        if n_scouts not in results_by_scouts:
            results_by_scouts[n_scouts] = []
        results_by_scouts[n_scouts].append(result["eval_result"]["success_rate"])

    scouts = sorted(results_by_scouts.keys())
    means = [np.mean(results_by_scouts[n]) for n in scouts]
    stds = [np.std(results_by_scouts[n]) if len(results_by_scouts[n]) > 1 else 0 for n in scouts]

    fig, ax = plt.subplots(figsize=(8, 6))

    ax.errorbar(
        scouts,
        means,
        yerr=stds,
        marker="o",
        markersize=10,
        linewidth=2,
        capsize=5,
        color="darkorange",
    )

    ax.set_xlabel("Number of Scouts", fontsize=12)
    ax.set_ylabel("Success Rate (Evaluation)", fontsize=12)
    ax.set_title("Performance vs Number of Scouts", fontsize=14)
    ax.set_xticks(scouts)
    ax.set_ylim(0, 1)
    ax.grid(True, alpha=0.3)

    # Add annotations
    for n, mean, std in zip(scouts, means, stds):
        ax.annotate(
            f"{mean:.1%}",
            xy=(n, mean),
            xytext=(5, 10),
            textcoords="offset points",
            fontsize=10,
        )

    plt.tight_layout()
    output_file = output_dir / "scouts_scaling.png"
    plt.savefig(output_file, dpi=150, bbox_inches="tight")
    print(f"Saved: {output_file}")
    plt.close()


def generate_all_plots(results_file: Path, output_dir: Path):
    """Generate all plots from results."""
    data = load_results(results_file)
    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"Generating plots from: {results_file}")
    print(f"Output directory: {output_dir}")

    plot_success_rate_comparison(data, output_dir)
    plot_learning_curves(data, output_dir)
    plot_scouts_scaling(data, output_dir)

    print("\nAll plots generated successfully!")


def main():
    parser = argparse.ArgumentParser(description="Generate plots from experiment results")
    parser.add_argument(
        "--results",
        type=str,
        default="experiments/results/results.json",
        help="Path to results.json",
    )
    parser.add_argument(
        "--output-dir",
        type=str,
        default="experiments/results/plots",
        help="Output directory for plots",
    )

    args = parser.parse_args()

    generate_all_plots(Path(args.results), Path(args.output_dir))


if __name__ == "__main__":
    main()
