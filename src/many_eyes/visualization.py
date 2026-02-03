"""Visualization utilities for many-eyes learning."""

import sys
import time
from dataclasses import dataclass

from many_eyes.core import Trajectory
from many_eyes.envs.sparse_grid import SparseGridWorld
from many_eyes.protocols import Policy


# ANSI color codes
class Colors:
    RESET = "\033[0m"
    BOLD = "\033[1m"
    RED = "\033[91m"
    GREEN = "\033[92m"
    YELLOW = "\033[93m"
    BLUE = "\033[94m"
    MAGENTA = "\033[95m"
    CYAN = "\033[96m"
    WHITE = "\033[97m"
    BG_GREEN = "\033[42m"
    BG_RED = "\033[41m"


def clear_screen():
    """Clear terminal screen."""
    print("\033[2J\033[H", end="")


def move_cursor(row: int, col: int):
    """Move cursor to position."""
    print(f"\033[{row};{col}H", end="")


@dataclass
class GridVisualizer:
    """Visualize grid world exploration in terminal."""

    env: SparseGridWorld
    delay: float = 0.1

    def render_grid(
        self,
        agent_pos: tuple[int, int],
        visited: set[tuple[int, int]] | None = None,
        show_path: bool = True,
    ) -> str:
        """Render grid with agent position and visited cells."""
        visited = visited or set()
        lines = []

        # Header
        lines.append(f"  {''.join(str(c) for c in range(self.env.size))}")

        for row in range(self.env.size):
            line = f"{row} "
            for col in range(self.env.size):
                pos = (row, col)

                if pos == agent_pos:
                    line += f"{Colors.BOLD}{Colors.BLUE}A{Colors.RESET}"
                elif pos == self.env.goal:
                    line += f"{Colors.BOLD}{Colors.GREEN}G{Colors.RESET}"
                elif pos in self.env.walls:
                    line += f"{Colors.RED}#{Colors.RESET}"
                elif show_path and pos in visited:
                    line += f"{Colors.CYAN}.{Colors.RESET}"
                else:
                    line += " "
            lines.append(line)

        return "\n".join(lines)

    def animate_trajectory(self, trajectory: Trajectory, title: str = ""):
        """Animate a trajectory in the terminal."""
        visited = set()

        clear_screen()

        for i, transition in enumerate(trajectory):
            # Decode position from one-hot state
            state_idx = int(transition.state.argmax())
            row = state_idx // self.env.size
            col = state_idx % self.env.size
            pos = (row, col)
            visited.add(pos)

            move_cursor(1, 1)

            if title:
                print(f"{Colors.BOLD}{title}{Colors.RESET}")
                print()

            print(self.render_grid(pos, visited))
            print()
            print(f"Step: {i + 1}/{len(trajectory)}")
            print(f"Position: ({row}, {col})")
            print(f"Action: {['UP', 'RIGHT', 'DOWN', 'LEFT'][transition.action]}")
            print(f"Reward: {transition.reward}")

            if transition.reward > 0:
                print(f"\n{Colors.GREEN}*** GOAL REACHED! ***{Colors.RESET}")

            sys.stdout.flush()
            time.sleep(self.delay)

    def show_policy(self, policy: Policy, title: str = "Learned Policy"):
        """Display the learned policy as arrows on the grid."""
        arrows = ["^", ">", "v", "<"]  # UP, RIGHT, DOWN, LEFT

        clear_screen()
        print(f"{Colors.BOLD}{title}{Colors.RESET}")
        print()
        print(f"  {''.join(str(c) for c in range(self.env.size))}")

        for row in range(self.env.size):
            line = f"{row} "
            for col in range(self.env.size):
                pos = (row, col)

                if pos == self.env.goal:
                    line += f"{Colors.GREEN}G{Colors.RESET}"
                elif pos in self.env.walls:
                    line += f"{Colors.RED}#{Colors.RESET}"
                else:
                    # Get policy action for this state
                    import numpy as np

                    state = np.zeros(self.env.size * self.env.size, dtype=np.float32)
                    state[row * self.env.size + col] = 1.0
                    action = policy(state)
                    line += f"{Colors.CYAN}{arrows[action]}{Colors.RESET}"
            print(line)

        print()
        print("Legend: ^ UP, > RIGHT, v DOWN, < LEFT")
        print(f"        {Colors.GREEN}G{Colors.RESET} Goal, {Colors.RED}#{Colors.RESET} Wall")


def print_training_progress(
    episode: int,
    total_episodes: int,
    reward: float,
    success_rate: float,
    loss: float,
    method: str = "",
):
    """Print a nicely formatted training progress line."""
    bar_width = 20
    progress = episode / total_episodes
    filled = int(bar_width * progress)
    bar = "=" * filled + ">" + " " * (bar_width - filled - 1)

    if success_rate > 0.5:
        status_color = Colors.GREEN
    elif success_rate > 0.1:
        status_color = Colors.YELLOW
    else:
        status_color = Colors.RED

    print(
        f"\r[{bar}] {episode:3d}/{total_episodes} | "
        f"R: {reward:6.1f} | "
        f"Success: {status_color}{success_rate:5.1%}{Colors.RESET} | "
        f"Loss: {loss:.4f}",
        end="",
    )

    if method:
        print(f" | {method}", end="")

    sys.stdout.flush()


def print_comparison_table(results: dict[str, dict]):
    """Print a comparison table of results."""
    print()
    header = f"{'Method':<20} {'Success Rate':>12} {'Mean Reward':>12} {'Std':>8}"
    print(f"{Colors.BOLD}{header}{Colors.RESET}")
    print("-" * 56)

    for method, data in results.items():
        success = data.get("success_rate", 0)
        mean_r = data.get("mean_reward", 0)
        std_r = data.get("std_reward", 0)

        color = Colors.GREEN if success > 0.5 else Colors.YELLOW if success > 0.1 else Colors.RED
        print(f"{method:<20} {color}{success:>11.1%}{Colors.RESET} {mean_r:>12.2f} {std_r:>8.2f}")

    print("-" * 56)
