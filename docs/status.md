# Project Status

## Current Status: Phase 2 In Progress

**Last Updated**: 2026-02-03

## Summary

Phase 1 complete, Phase 2 partially complete. The system now has:
- Proper evaluation (greedy policy, not exploration)
- Reproducible experiments with multiple seeds
- CLI visualization and demo
- Result plots and documentation
- Verified training that actually learns

## Progress Overview

| Phase | Status | Progress |
|-------|--------|----------|
| Phase 1: Foundation | Complete | 100% |
| Phase 2: Diversity | In Progress | 90% |
| Phase 3: Parallel | Not Started | 0% |
| Phase 4: Polish | Not Started | 0% |

## Latest Experimental Results

### Many-Eyes Scaling (7x7 grid, 75 episodes)

| Method | Success Rate |
|--------|-------------|
| Random baseline | 9% |
| Single scout | 0% |
| Many eyes (3) | 40% |
| Many eyes (5) | 60% |

### Diversity Experiment (7x7 grid, 100 episodes, 5 scouts each)

| Configuration | Success Rate |
|--------------|-------------|
| Random baseline | 7% |
| Homogeneous random | 20% |
| Homogeneous epsilon | 40% |
| Diverse mix | 40% |

**Finding**: Strategy quality matters more than diversity in simple environments.

See [results.md](results.md) for full details and reproduction instructions.

## Phase 2 Status: Diversity & Validation

### Completed
- [x] Evaluation module (greedy policy evaluation)
- [x] Visualization module (CLI grid display, policy arrows)
- [x] Reproducible experiment runner
- [x] Multi-seed experiments (statistical significance)
- [x] Result plotting (comparison, learning curves, scaling)
- [x] Results documentation (docs/results.md)
- [x] CLI demo with interactive visualization
- [x] Fixed DQN learning (step penalty for gradient signal)
- [x] Fair comparison (equal total environment steps)
- [x] CuriousScout (count-based novelty exploration)
- [x] OptimisticScout (optimistic initialization)
- [x] Diversity experiment (homogeneous vs diverse scouts)
- [x] Diversity results and plots

### Not Yet Started
- [ ] Weighted aggregation strategies
- [ ] Web UI demo

## Blockers

None currently.

## Recent Activity

| Date | Activity |
|------|----------|
| 2026-02-03 | Added CuriousScout, OptimisticScout, diversity experiment |
| 2026-02-03 | Phase 2: Added evaluation, experiments, CLI demo, plots, results.md |
| 2026-02-03 | Fixed DQN learning with step penalty |
| 2026-02-03 | Phase 1 complete: core system, tests, demo |
| 2026-02-01 | Documentation and initial structure |

## Key Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Tests | 56 passing | - |
| Demo environments | 1 (SparseGridWorld) | 1+ |
| Scout strategies | 4 (Random, EpsilonGreedy, Curious, Optimistic) | 4+ |
| Reproducible results | Yes | Yes |
| CLI visualization | Yes | Yes |

## Reproduction

```bash
# Setup
uv venv .venv && source .venv/bin/activate
uv pip install -e ".[dev]"

# Run tests
pytest

# Run CLI demo
python experiments/cli_demo.py

# Run full experiment
python experiments/run_experiment.py --episodes 75 --scouts 1 3 5

# Generate plots
python experiments/plot_results.py
```

## Links

- [Results](results.md) - Experimental results
- [Development Plan](plan.md)
- [Architecture](architecture.md)
- [Design](design.md)
- [PRD](prd.md)
