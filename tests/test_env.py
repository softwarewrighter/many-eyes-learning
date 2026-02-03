"""Tests for environments."""

from many_eyes.envs.sparse_grid import SparseGridWorld


def test_grid_creation():
    """Test creating a grid world."""
    env = SparseGridWorld(size=5)

    assert env.size == 5
    assert env.n_actions == 4
    assert env.state_shape == (25,)
    assert env.goal == (4, 4)


def test_grid_reset():
    """Test resetting the environment."""
    env = SparseGridWorld(size=3)
    state = env.reset()

    assert state.shape == (9,)
    assert state[0] == 1.0  # Agent at (0, 0)
    assert state.sum() == 1.0  # One-hot


def test_grid_step_right():
    """Test stepping right."""
    env = SparseGridWorld(size=3, step_penalty=0.0)  # No penalty for this test
    env.reset()

    # Move right
    next_state, reward, done, info = env.step(1)

    assert info["position"] == (0, 1)
    assert reward == 0.0
    assert not done


def test_grid_step_down():
    """Test stepping down."""
    env = SparseGridWorld(size=3, step_penalty=0.0)  # No penalty for this test
    env.reset()

    # Move down
    next_state, reward, done, info = env.step(2)

    assert info["position"] == (1, 0)
    assert reward == 0.0


def test_grid_reach_goal():
    """Test reaching the goal."""
    env = SparseGridWorld(size=2)
    env.reset()

    # In 2x2 grid, goal is at (1, 1)
    # From (0, 0): right to (0, 1), down to (1, 1)
    env.step(1)  # right
    _, reward, done, info = env.step(2)  # down

    assert info["position"] == (1, 1)
    assert reward == 1.0
    assert done
    assert info["reached_goal"]


def test_grid_wall_collision():
    """Test wall collision."""
    env = SparseGridWorld(size=3, walls={(0, 1)})
    env.reset()

    # Try to move right into wall
    next_state, _, _, info = env.step(1)

    # Should stay at (0, 0)
    assert info["position"] == (0, 0)


def test_grid_boundary():
    """Test boundary collision."""
    env = SparseGridWorld(size=3)
    env.reset()

    # Try to move up from (0, 0)
    _, _, _, info = env.step(0)
    assert info["position"] == (0, 0)

    # Try to move left from (0, 0)
    _, _, _, info = env.step(3)
    assert info["position"] == (0, 0)


def test_grid_max_steps():
    """Test max steps termination."""
    env = SparseGridWorld(size=5, max_steps=3)
    env.reset()

    # Take 3 steps without reaching goal
    for _ in range(3):
        _, _, done, _ = env.step(0)  # up (stays at 0,0)

    assert done


def test_grid_copy():
    """Test environment copy is independent."""
    env = SparseGridWorld(size=3)
    env.reset()
    env.step(1)  # Move right

    env_copy = env.copy()

    # Original moves down
    env.step(2)

    # Copy should still be at (0, 1)
    assert env._pos == (1, 1)
    assert env_copy._pos == (0, 1)


def test_grid_render():
    """Test rendering."""
    env = SparseGridWorld(size=3)
    env.reset()

    rendered = env.render()

    assert "A" in rendered  # Agent
    assert "G" in rendered  # Goal


def test_grid_with_obstacles():
    """Test creating grid with random obstacles."""
    env = SparseGridWorld.with_obstacles(size=5, seed=42)

    assert len(env.walls) > 0
    assert (0, 0) not in env.walls  # Start not blocked
    assert (4, 4) not in env.walls  # Goal not blocked


def test_grid_state_encoding():
    """Test one-hot state encoding."""
    env = SparseGridWorld(size=3)
    env.reset()

    # Move to (1, 1)
    env.step(1)  # right to (0, 1)
    state, _, _, _ = env.step(2)  # down to (1, 1)

    # (1, 1) in 3x3 grid is index 1*3 + 1 = 4
    assert state[4] == 1.0
    assert state.sum() == 1.0
