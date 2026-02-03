# many-eyes-learning

**Structured exploration for better learning under sparse rewards**

---

Learning often fails not because models are slow, but because they see too little.

In sparse-reward problems, useful feedback is rare. A single learner exploring alone is likely to miss it entirely. When that happens, learning stalls—not due to lack of compute, but due to lack of information.

This project explores a simple idea:

> **Put many eyes on the problem.**

Instead of relying on one explorer, we use multiple exploratory passes—*many eyes*—to gather diverse information during training. What those eyes discover is then combined to improve learning outcomes.

The goal is **not speed**.  
The goal is **better learning through better information**.

---

## What “many eyes” means here

In this project, **many eyes refers to training-time exploration**, not inference-time assistance.

- Each “eye” is an exploratory run (a *scout*) with its own bias or exploration strategy.
- Scouts gather experience in environments where rewards are sparse.
- Their discoveries are aggregated to improve a shared learner.

Parallel execution is optional. Scouts can run sequentially or concurrently—the benefit comes from **diversity of exploration**, not wall-clock speed.

---

## What this project is (and is not)

**This project is:**
- About improving learning outcomes in sparse-reward settings
- About spending more computation *intelligently*, not faster
- About exploration as an information problem
- About simple, reproducible demos that run on modest hardware

**This project is not:**
- A benchmark shootout
- A GPU optimization project
- A claim about training speedups
- A requirement for parallel or distributed systems

---

## Why this matters

Sparse rewards create an information bottleneck. When feedback is rare, gradients are weak and learning becomes fragile or impossible.

Adding more exploration—*more eyes*—helps because it:
- reduces blind spots,
- increases coverage of the problem space,
- uncovers rare but valuable learning signal.

Better information leads to better learning.

---

## Results

On a 7x7 sparse-reward grid (random baseline ~9% success):

![Success Rate Comparison](docs/images/success_rate_comparison.png)

| Method | Success Rate |
|--------|-------------|
| Single scout | 0% (fails to learn) |
| Many eyes (3 scouts) | 40% |
| Many eyes (5 scouts) | 60% |

See [Results](docs/results.md) for full analysis and reproduction instructions.

---

## Project structure (high-level)

This repository contains:
- Minimal environments with sparse rewards
- Sequential and parallel exploration modes
- Aggregation strategies for scout experience
- Visualizations that reconstruct "many eyes" exploration
- Reproducible experiments demonstrating improved learning outcomes

The same code paths can run:
- sequentially on a laptop, or
- in parallel on larger systems

The results should be comparable - the difference is only wall-clock time.

---

## Documentation

- [ELI5](docs/eli5.md) - Simple explanation of the papers and what this repo demonstrates
- [Results](docs/results.md) - Experimental results and reproduction instructions
- [Architecture](docs/architecture.md) - System components and data flow
- [PRD](docs/prd.md) - Product requirements and goals
- [Design](docs/design.md) - Implementation details and design decisions
- [Development Plan](docs/plan.md) - Phased development roadmap
- [Status](docs/status.md) - Current progress and next steps

Additional documentation:
- [AI Agent Instructions](docs/ai_agent_instructions.md) - Guidelines for AI coding agents
- [Development Process](docs/process.md) - TDD workflow and quality gates
- [Tools](docs/tools.md) - Recommended development tools

---

## Quick Start

```bash
# Setup
uv venv .venv
source .venv/bin/activate
uv pip install -e ".[dev]"

# Run interactive CLI demo
python experiments/cli_demo.py

# Run reproducible experiments
python experiments/run_experiment.py --episodes 75 --scouts 1 3 5

# Generate plots
python experiments/plot_results.py
```

---

## Relationship to research

This project is inspired by research on:
- structured exploration in sparse-reward environments,
- multi-policy exploration (e.g. scout-based methods),
- intrinsic motivation and information-driven learning.

The focus here is **clarity and intuition**, not exact reproduction of any single paper.

---

## Guiding principle

> *The problem isn’t that learning is slow.  
The problem is that learning is blind.*

Many eyes make learning see.

