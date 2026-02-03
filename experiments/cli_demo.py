#!/usr/bin/env python3
"""
Interactive CLI demo of many-eyes learning.

Shows:
1. The grid world environment
2. Training progress with live updates
3. Visualization of learned policy
4. Animated playback of agent behavior
"""

import argparse
import sys

from many_eyes.envs.sparse_grid import SparseGridWorld
from many_eyes.evaluation import evaluate_policy
from many_eyes.learner.dqn import DQNLearner
from many_eyes.scouts.base import RandomScout
from many_eyes.scouts.epsilon import EpsilonGreedyScout
from many_eyes.trainer import Trainer, TrainingConfig
from many_eyes.visualization import (
    Colors,
    GridVisualizer,
    clear_screen,
    print_comparison_table,
    print_training_progress,
)


def demo_environment():
    """Show the environment."""
    print(f"\n{Colors.BOLD}=== ENVIRONMENT ==={Colors.RESET}\n")

    env = SparseGridWorld(size=5, max_steps=50)
    viz = GridVisualizer(env)

    print("Sparse Grid World (5x5)")
    print("- Agent starts at (0,0)")
    print("- Goal is at (4,4)")
    print("- Reward: +1 at goal, 0 elsewhere")
    print()
    print(viz.render_grid((0, 0)))
    print()

    return env


def demo_random_exploration(env: SparseGridWorld):
    """Show random exploration."""
    print(f"\n{Colors.BOLD}=== RANDOM EXPLORATION ==={Colors.RESET}\n")
    print("Watch a random agent explore the grid...")
    input("Press Enter to start...")

    viz = GridVisualizer(env, delay=0.15)
    scout = RandomScout(seed=42)
    trajectories = scout.explore(env.copy(), steps=30)

    if trajectories:
        viz.animate_trajectory(trajectories[0], "Random Exploration")

    reached = "reached" if trajectories[0].reached_goal else "did NOT reach"
    print(f"\n\nRandom agent {reached} the goal")


def train_and_visualize(env: SparseGridWorld, n_scouts: int, episodes: int = 50):
    """Train and show progress."""
    method = "SINGLE SCOUT" if n_scouts == 1 else f"MANY EYES ({n_scouts} scouts)"
    print(f"\n{Colors.BOLD}=== TRAINING: {method} ==={Colors.RESET}\n")

    # Create learner
    learner = DQNLearner(
        state_dim=env.size * env.size,
        n_actions=env.n_actions,
        seed=42,
    )

    # Create scouts
    scouts = []
    for i in range(n_scouts):
        if i == 0:
            scouts.append(RandomScout(scout_id=f"random_{i}", seed=42 + i))
        else:
            epsilon = 0.1 + 0.2 * (i / n_scouts)
            scouts.append(EpsilonGreedyScout(epsilon=epsilon, scout_id=f"eps_{i}", seed=42 + i))

    # Training config
    steps_per_scout = 100 // n_scouts
    config = TrainingConfig(
        episodes=episodes,
        steps_per_episode=steps_per_scout,
        learning_steps=20,
        update_scouts_every=5,
        verbose=False,
    )

    # Note: We manually run training loop for progress display
    _ = Trainer(env=env, scouts=scouts, learner=learner, config=config)

    # Train with progress
    print(f"Training for {episodes} episodes...")
    print()

    for episode in range(episodes):
        all_trajectories = []
        for scout in scouts:
            env_copy = env.copy()
            trajectories = scout.explore(env_copy, steps_per_scout)
            all_trajectories.extend(trajectories)

        metrics = learner.update(all_trajectories, config.learning_steps)

        episode_reward = sum(t.total_reward for t in all_trajectories)
        successes = sum(1 for t in all_trajectories if t.reached_goal)
        success_rate = successes / len(all_trajectories) if all_trajectories else 0

        if (episode + 1) % 5 == 0:
            policy = learner.get_policy()
            for scout in scouts:
                scout.set_policy(policy)

        print_training_progress(
            episode + 1, episodes, episode_reward, success_rate, metrics.get("loss", 0), method
        )

    print("\n")

    return learner


def evaluate_and_show(env: SparseGridWorld, learner: DQNLearner, method: str):
    """Evaluate and visualize learned policy."""
    print(f"\n{Colors.BOLD}=== EVALUATION: {method} ==={Colors.RESET}\n")

    policy = learner.get_policy()

    # Show learned policy
    viz = GridVisualizer(env)
    viz.show_policy(policy, f"Learned Policy ({method})")
    print()

    # Evaluate
    result = evaluate_policy(policy, env.copy(), n_episodes=50, max_steps=50)

    print("\nEvaluation over 50 episodes (greedy policy):")
    color = Colors.GREEN if result.success_rate > 0.5 else Colors.RED
    print(f"  Success rate: {color}{result.success_rate:.1%}{Colors.RESET}")
    print(f"  Mean reward:  {result.mean_reward:.2f}")
    print(f"  Mean steps:   {result.mean_steps:.1f}")

    return result


def demo_learned_behavior(env: SparseGridWorld, learner: DQNLearner, method: str):
    """Animate the learned policy."""
    print(f"\n{Colors.BOLD}=== LEARNED BEHAVIOR ==={Colors.RESET}\n")
    print(f"Watch the {method} policy navigate the grid...")
    input("Press Enter to start...")

    policy = learner.get_policy()
    viz = GridVisualizer(env, delay=0.3)

    # Run one episode
    from many_eyes.core import Trajectory, Transition

    trajectory = Trajectory()
    state = env.reset()

    for _ in range(50):
        action = policy(state)
        next_state, reward, done, _ = env.step(action)
        t = Transition(state=state, action=action, reward=reward, next_state=next_state, done=done)
        trajectory.add(t)
        state = next_state
        if done:
            break

    viz.animate_trajectory(trajectory, f"Learned Policy ({method})")

    status = "REACHED" if trajectory.reached_goal else "did NOT reach"
    print(f"\n\nAgent {status} the goal in {len(trajectory)} steps")


def run_comparison_demo():
    """Run a side-by-side comparison."""
    clear_screen()
    print(f"{Colors.BOLD}{'=' * 60}")
    print("    MANY-EYES LEARNING: INTERACTIVE DEMO")
    print(f"{'=' * 60}{Colors.RESET}\n")

    env = demo_environment()
    input("\nPress Enter to continue...")

    # Demo random exploration
    clear_screen()
    demo_random_exploration(env)
    input("\nPress Enter to continue...")

    # Train single scout
    clear_screen()
    learner_single = train_and_visualize(env, n_scouts=1, episodes=50)
    input("\nPress Enter to continue...")

    # Train many eyes
    clear_screen()
    learner_many = train_and_visualize(env, n_scouts=5, episodes=50)
    input("\nPress Enter to continue...")

    # Evaluate both
    clear_screen()
    result_single = evaluate_and_show(env, learner_single, "Single Scout")
    input("\nPress Enter to see Many Eyes evaluation...")

    clear_screen()
    result_many = evaluate_and_show(env, learner_many, "Many Eyes (5)")

    # Summary
    print(f"\n{Colors.BOLD}=== COMPARISON ==={Colors.RESET}")
    print_comparison_table(
        {
            "Single Scout": result_single.to_dict(),
            "Many Eyes (5)": result_many.to_dict(),
        }
    )

    # Offer to show learned behavior
    print()
    response = input("Watch learned behavior? [y/N]: ").strip().lower()
    if response == "y":
        clear_screen()
        demo_learned_behavior(env, learner_many, "Many Eyes")

    print(f"\n{Colors.BOLD}Demo complete!{Colors.RESET}")
    print("Run 'python experiments/run_experiment.py' for full reproducible experiments.")


def main():
    parser = argparse.ArgumentParser(description="Interactive CLI demo")
    parser.add_argument("--quick", action="store_true", help="Quick mode")
    parser.parse_args()  # Parse but currently unused

    try:
        run_comparison_demo()
    except KeyboardInterrupt:
        print(f"\n\n{Colors.YELLOW}Demo interrupted.{Colors.RESET}")
        sys.exit(0)


if __name__ == "__main__":
    main()
