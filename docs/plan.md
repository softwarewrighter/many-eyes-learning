# Development Plan

## Current Phase: Phase 2 (90% complete)

This document tracks the development plan and progress for the Many-Eyes Learning project.

## Phase 1: Foundation (MVP) - COMPLETE

### Objectives
- Establish core abstractions
- Implement basic exploration
- Create minimal demo

### Tasks

#### 1.1 Project Setup
- [x] Initialize Python project structure
- [x] Set up testing framework (pytest)
- [x] Configure linting (ruff)
- [ ] Create basic CI workflow

#### 1.2 Core Abstractions
- [x] Define Scout protocol
- [x] Define Trajectory dataclass
- [x] Define Environment protocol
- [x] Define Aggregator protocol
- [x] Define Learner protocol

#### 1.3 Basic Scout Implementation
- [x] Implement RandomScout (baseline)
- [x] Implement EpsilonGreedyScout
- [x] Write scout unit tests

#### 1.4 Experience Buffer
- [x] Implement ReplayBuffer
- [x] Implement trajectory storage
- [x] Write buffer unit tests

#### 1.5 Simple Environment
- [x] Implement SparseGridWorld
- [x] Make environment copyable
- [x] Write environment tests
- [x] Add visualization (text-based)

#### 1.6 Basic Learner
- [x] Implement DQN learner (with target network)
- [x] Test with single scout
- [x] Verify learning on sparse-reward grid

#### 1.7 Integration
- [x] Wire scouts + buffer + learner
- [x] Sequential execution mode
- [x] Basic training loop
- [x] Integration tests

#### 1.8 First Demo
- [x] Create demo script
- [x] Show single vs. many scouts
- [x] Generate first learning curve

## Phase 2: Diversity - IN PROGRESS (90%)

### Objectives
- Multiple exploration strategies
- Aggregation strategies
- Better visualization

### Tasks

#### 2.1 Additional Scouts
- [x] Implement CuriousScout (count-based)
- [x] Implement OptimisticScout
- [ ] Implement BoltzmannScout
- [x] Tests for each scout

#### 2.2 Heterogeneous Configuration
- [x] Support mixed scout types
- [ ] Configuration file format
- [ ] Config validation

#### 2.3 Aggregation Strategies
- [x] Implement SimpleAggregator
- [ ] Implement RewardWeightedAggregator
- [ ] Implement NoveltyWeightedAggregator
- [x] Tests for aggregator

#### 2.4 Visualization
- [x] CLI grid visualization with policy arrows
- [x] Learning curve comparison plots
- [ ] Scout behavior animation
- [x] Matplotlib-based output

#### 2.5 Experiments
- [x] Design comparison experiments
- [x] Run scout diversity experiments
- [ ] Run aggregation comparison
- [x] Document findings in results.md

## Immediate Next Steps

These are the recommended next actions:

### Option A: Finish Phase 2 (Recommended)
1. **Weighted aggregation** - Implement RewardWeightedAggregator to prioritize high-reward trajectories
2. **Web UI demo** - Simple browser-based visualization using Rust/Yew/WASM
3. **BoltzmannScout** - Temperature-based action selection

### Option B: Start Phase 3
1. **Parallel execution** - Run scouts concurrently
2. **Larger experiments** - 10x10 grid, 10+ scouts
3. **Performance analysis** - Compare sequential vs parallel

### Option C: Content Creation
1. **YouTube Short** - Quick visual of scouts finding goal
2. **Blog post** - Explain papers and results
3. **Jupyter notebook** - Interactive tutorial

## Phase 3: Parallel Execution

### Objectives
- Parallel execution mode
- Performance tooling
- Extended experiments

### Tasks

#### 3.1 Parallel Mode
- [ ] Implement parallel scout execution
- [ ] Thread/process pool management
- [ ] Synchronization points

#### 3.2 Consistency
- [ ] Compare sequential vs parallel results
- [ ] Handle non-determinism
- [ ] Document expectations

#### 3.3 Performance Tools
- [ ] Timing instrumentation
- [ ] Memory profiling
- [ ] Scalability analysis

#### 3.4 Extended Experiments
- [ ] Larger grid worlds
- [ ] More scouts (10, 20, 50)
- [ ] Strategy combinations

## Phase 4: Polish

### Objectives
- Complete documentation
- Reproducibility package
- Example notebooks

### Tasks

#### 4.1 Documentation
- [ ] Complete API documentation
- [ ] Tutorial notebook
- [ ] Architecture guide
- [ ] Troubleshooting guide

#### 4.2 Reproducibility
- [ ] Experiment scripts
- [ ] Result caching
- [ ] Figure generation
- [ ] Version pinning

#### 4.3 Examples
- [ ] Quick start example
- [ ] Custom scout example
- [ ] Custom environment example
- [ ] Visualization examples

#### 4.4 Release Preparation
- [ ] Clean up codebase
- [ ] Final testing
- [ ] Release notes
- [ ] Publish

## Backlog

Items that may be added in future phases:

- Continuous action space support
- PPO learner
- More complex environments
- Learned exploration strategies
- Distributed execution
- Hyperparameter search
- Benchmark comparisons

## Timeline

This is a research/educational project with no hard deadlines. Progress is tracked in [status.md](status.md).

## Notes

### Design Principles (Reminders)
- Clarity over performance
- Sequential first, parallel optional
- Minimal dependencies
- Reproducibility always

### Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| - | Python as primary language | Accessibility, ecosystem |
| - | DQN as first learner | Simplicity, well understood |
| - | Grid world as first env | Easy to visualize, sparse rewards |

## References

- [PRD](prd.md) - Product requirements
- [Architecture](architecture.md) - System design
- [Design](design.md) - Implementation details
- [Status](status.md) - Current progress
