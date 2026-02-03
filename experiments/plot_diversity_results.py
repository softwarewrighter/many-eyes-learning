#!/usr/bin/env python3
"""Generate plots from diversity experiment results."""

import argparse
import json
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np


def load_results(results_file: Path) -> dict:
    """Load results from JSON file."""
    with open(results_file) as f:
        return json.load(f)


def plot_diversity_comparison(data: dict, output_dir: Path):
    """Plot diversity comparison bar chart."""
    results_by_config = {}

    for result in data["results"]:
        config = result["config_name"]
        if config not in results_by_config:
            results_by_config[config] = []
        results_by_config[config].append(result["eval_result"]["success_rate"])

    configs = ["random_baseline", "homogeneous_random", "homogeneous_epsilon", "diverse"]
    labels = ["Random\nBaseline", "Homogeneous\nRandom", "Homogeneous\nEpsilon", "Diverse\nMix"]
    means = []
    stds = []

    for config in configs:
        if config in results_by_config:
            rates = results_by_config[config]
            means.append(np.mean(rates))
            stds.append(np.std(rates) if len(rates) > 1 else 0)
        else:
            means.append(0)
            stds.append(0)

    fig, ax = plt.subplots(figsize=(10, 6))

    colors = ["gray", "steelblue", "forestgreen", "darkorange"]
    bars = ax.bar(labels, means, yerr=stds, capsize=5, color=colors, alpha=0.8)

    ax.set_ylabel("Success Rate", fontsize=12)
    ax.set_title("Scout Configuration Comparison (5 Scouts Each)", fontsize=14)
    ax.set_ylim(0, 1)

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
    output_file = output_dir / "diversity_comparison.png"
    plt.savefig(output_file, dpi=150, bbox_inches="tight")
    print(f"Saved: {output_file}")
    plt.close()


def plot_learning_curves_diversity(data: dict, output_dir: Path):
    """Plot learning curves for each configuration."""
    fig, ax = plt.subplots(figsize=(10, 6))

    results_by_config = {}
    for result in data["results"]:
        config = result["config_name"]
        if config == "random_baseline":
            continue
        if config not in results_by_config:
            results_by_config[config] = []
        if result["training_history"]:
            results_by_config[config].append(result["training_history"])

    colors = {
        "homogeneous_random": "steelblue",
        "homogeneous_epsilon": "forestgreen",
        "diverse": "darkorange",
    }

    for config, histories in results_by_config.items():
        if not histories:
            continue
        all_success = np.array([h["success_rates"] for h in histories])
        mean_success = np.mean(all_success, axis=0)
        std_success = np.std(all_success, axis=0)
        episodes = range(1, len(mean_success) + 1)

        color = colors.get(config, "gray")
        label = config.replace("_", " ").title()
        ax.plot(episodes, mean_success, label=label, color=color, linewidth=2)
        ax.fill_between(
            episodes,
            mean_success - std_success,
            mean_success + std_success,
            alpha=0.2,
            color=color,
        )

    ax.set_xlabel("Episode", fontsize=12)
    ax.set_ylabel("Training Success Rate", fontsize=12)
    ax.set_title("Learning Progress by Configuration", fontsize=14)
    ax.legend()
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    output_file = output_dir / "diversity_learning_curves.png"
    plt.savefig(output_file, dpi=150, bbox_inches="tight")
    print(f"Saved: {output_file}")
    plt.close()


def main():
    parser = argparse.ArgumentParser(description="Plot diversity experiment results")
    parser.add_argument(
        "--results",
        default="experiments/results/diversity_results.json",
    )
    parser.add_argument(
        "--output-dir",
        default="experiments/results/plots",
    )

    args = parser.parse_args()
    results_file = Path(args.results)
    output_dir = Path(args.output_dir)

    if not results_file.exists():
        print(f"Error: {results_file} not found")
        print("Run diversity experiment first:")
        print("  python experiments/run_diversity_experiment.py")
        return

    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"Generating plots from: {results_file}")
    data = load_results(results_file)

    plot_diversity_comparison(data, output_dir)
    plot_learning_curves_diversity(data, output_dir)

    print("\nDone!")


if __name__ == "__main__":
    main()
