# Development Plan

## Current Phase: Foundation

This document tracks the development plan and progress for the Many-Eyes Learning project.

## Phase 1: Foundation (MVP)

### Objectives
- Establish core abstractions
- Implement basic exploration
- Create minimal demo

### Tasks

#### 1.1 Project Setup
- [ ] Initialize Python project structure
- [ ] Set up testing framework (pytest)
- [ ] Configure linting (ruff/black)
- [ ] Create basic CI workflow

#### 1.2 Core Abstractions
- [ ] Define Scout protocol
- [ ] Define Trajectory dataclass
- [ ] Define Environment protocol
- [ ] Define Aggregator protocol
- [ ] Define Learner protocol

#### 1.3 Basic Scout Implementation
- [ ] Implement RandomScout (baseline)
- [ ] Implement EpsilonGreedyScout
- [ ] Write scout unit tests

#### 1.4 Experience Buffer
- [ ] Implement ReplayBuffer
- [ ] Implement trajectory storage
- [ ] Write buffer unit tests

#### 1.5 Simple Environment
- [ ] Implement SparseGridWorld
- [ ] Make environment copyable
- [ ] Write environment tests
- [ ] Add visualization (text-based)

#### 1.6 Basic Learner
- [ ] Implement DQN learner
- [ ] Test with single scout
- [ ] Verify learning on dense-reward grid

#### 1.7 Integration
- [ ] Wire scouts + buffer + learner
- [ ] Sequential execution mode
- [ ] Basic training loop
- [ ] Integration tests

#### 1.8 First Demo
- [ ] Create demo script
- [ ] Show single vs. many scouts
- [ ] Generate first learning curve

## Phase 2: Diversity

### Objectives
- Multiple exploration strategies
- Aggregation strategies
- Better visualization

### Tasks

#### 2.1 Additional Scouts
- [ ] Implement CuriousScout (count-based)
- [ ] Implement OptimisticScout
- [ ] Implement BoltzmannScout
- [ ] Tests for each scout

#### 2.2 Heterogeneous Configuration
- [ ] Support mixed scout types
- [ ] Configuration file format
- [ ] Config validation

#### 2.3 Aggregation Strategies
- [ ] Implement SimpleAggregator
- [ ] Implement RewardWeightedAggregator
- [ ] Implement NoveltyWeightedAggregator
- [ ] Tests for each aggregator

#### 2.4 Visualization
- [ ] Coverage map visualization
- [ ] Learning curve comparison
- [ ] Scout behavior animation
- [ ] Matplotlib-based output

#### 2.5 Experiments
- [ ] Design comparison experiments
- [ ] Run scout diversity experiments
- [ ] Run aggregation comparison
- [ ] Document findings

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
