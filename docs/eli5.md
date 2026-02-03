# ELI5: Many-Eyes Learning

*Explain Like I'm 5*

## The Problem: Finding Needles in Haystacks

Imagine you're a robot trying to learn how to solve a maze. The only feedback you get is:
- **"You won!"** when you reach the goal
- **Nothing** the rest of the time

This is called a **sparse reward** problem. It's brutal because:
- You stumble around randomly for ages
- You finally reach the goal once by accident
- But you have no idea which of your 1000 steps actually mattered

Most of your training time is wasted on uninformative "nothing happened" experiences.

## The Insight: Many Eyes Are Better Than One

What if instead of one robot exploring, you had **five robots exploring at the same time**?

- Robot 1 wanders north
- Robot 2 wanders south
- Robot 3 follows walls
- Robot 4 tries unexplored areas
- Robot 5 is optimistic about new actions

Now you have 5x the chance of someone finding the goal. When Robot 3 finds it, **all five robots learn from that discovery**.

This is the core idea: **better information (from diverse exploration) leads to better learning**.

## The Two Papers

This repo is inspired by two recent papers that tackle sparse rewards:

### Paper 1: IRPO (Intrinsic Reward Policy Optimization)

**The Problem**: In sparse reward RL, gradients are useless most of the time (no reward = no learning signal).

**The Solution**: Use multiple "scout" policies that explore using *intrinsic* rewards (curiosity, novelty, optimism). When scouts discover *extrinsic* rewards (the real goal), route that information back to update a base policy.

```
┌─────────────────────────────────────────────────┐
│                                                 │
│   Scout 1 (curious)  ──┐                        │
│   Scout 2 (random)   ──┼──► Aggregate ──► Base  │
│   Scout 3 (optimist) ──┘    findings     Policy │
│                                                 │
└─────────────────────────────────────────────────┘
```

**Key insight**: Intrinsic rewards keep scouts exploring. Extrinsic rewards from lucky discoveries train the real policy.

### Paper 2: Reagent (Reasoning Reward Model for Agents)

**The Problem**: LLM agents doing multi-step tasks only get "did you succeed?" at the end. No feedback on intermediate reasoning.

**The Solution**: Train a "reward model" that scores agent trajectories with:
- A reasoning trace (why this score?)
- A critique (what went wrong?)
- A scalar score (how good overall?)

**Key insight**: Dense, structured feedback helps even when the environment only gives sparse signals.

## What This Repo Demonstrates

We implement the "scouts" concept in a simple grid world:

### Environment
- 7x7 grid, start at (0,0), goal at (6,6)
- Reward: +1 at goal, -0.01 per step (sparse!)
- Random agent succeeds ~7-9% of the time

### Experiment 1: More Scouts = Better Learning

| Configuration | Success Rate |
|--------------|--------------|
| Random baseline | 9% |
| 1 scout (single learner) | 0% |
| 3 scouts (many-eyes) | 40% |
| 5 scouts (many-eyes) | 60% |

**Result**: Single scout fails completely. Many scouts succeed.

With limited training, a single explorer rarely finds the goal, so the learner has nothing useful to learn from. Multiple scouts have a much higher chance of discovering successful paths.

### Experiment 2: Does Diversity Matter?

We tested whether *different* exploration strategies help more than just *more* explorers:

| Configuration | Success Rate |
|--------------|--------------|
| 5 random scouts | 20% |
| 5 epsilon-greedy scouts | 40% |
| Diverse mix (random + epsilon + curious + optimistic) | 40% |

**Result**: Strategy quality matters more than diversity (in simple environments).

Epsilon-greedy is just better than pure random. Having a diverse mix doesn't beat having 5 good epsilon-greedy scouts. Diversity may matter more in complex environments with multiple local optima.

## The Scout Strategies

### RandomScout
Pure random actions. Simple baseline.

### EpsilonGreedyScout
Mostly follows the learned policy, but takes random actions ε% of the time. Balances exploitation and exploration.

### CuriousScout
Prefers visiting states it hasn't seen before. Uses "count-based" intrinsic motivation:

```
bonus = scale / sqrt(visit_count + 1)
```

Novel states get higher bonuses, driving exploration.

### OptimisticScout
Starts with high expectations for untried actions. As actions are tried, optimism decays. This naturally explores all options before settling.

## Why This Matters

1. **Sample efficiency**: Same total environment interactions, better outcomes
2. **Sparse rewards are everywhere**: Real-world tasks rarely give continuous feedback
3. **Scalable**: Add more scouts on more compute for harder problems
4. **Composable**: Different scout strategies can be mixed and matched

## Try It Yourself

```bash
# Setup
uv venv .venv && source .venv/bin/activate
uv pip install -e ".[dev]"

# Run interactive demo
python experiments/cli_demo.py

# Run the many-eyes experiment
python experiments/run_experiment.py --scouts 1 3 5

# Run the diversity experiment
python experiments/run_diversity_experiment.py
```

## Links

**Papers**:
- [IRPO: Intrinsic Reward Policy Optimization](https://arxiv.org/abs/2601.21391) - Cho & Tran, UIUC
- [Reagent: Exploring Reasoning Reward Model for Agents](https://arxiv.org/abs/2601.22154) - Fan et al., CUHK/Meituan

**Code**:
- [IRPO GitHub](https://github.com/Mgineer117/IRPO)
- [Reagent GitHub](https://github.com/kxfan2002/Reagent)

## TL;DR

> Sparse rewards make learning hard because you rarely get useful feedback.
>
> Solution: Send out multiple scouts with different exploration strategies.
> When any scout finds success, everyone learns from it.
>
> More scouts = more discoveries = better learning.
