# Product Requirements Document (PRD)

## Problem Statement

Learning in sparse-reward environments often fails not because models are slow, but because they see too little. When useful feedback is rare, a single learner exploring alone is likely to miss it entirely. This creates an information bottleneck that makes learning fragile or impossible.

## Vision

Put many eyes on the problem. Instead of relying on one explorer, use multiple exploratory passes - many eyes - to gather diverse information during training. The goal is better learning through better information, not speed.

## Target Users

1. **Researchers** studying exploration in reinforcement learning
2. **Practitioners** working with sparse-reward environments
3. **Educators** teaching exploration strategies in RL

## Goals

### Primary Goals

1. **Demonstrate improved learning outcomes** in sparse-reward settings through structured exploration
2. **Provide clear, intuitive implementations** that prioritize understanding over optimization
3. **Enable reproducible experiments** on modest hardware

### Non-Goals

1. Benchmark shootouts against state-of-the-art methods
2. GPU optimization or distributed systems engineering
3. Claims about training speedups
4. Production-ready reinforcement learning framework

## Requirements

### Functional Requirements

#### FR1: Scout-Based Exploration

- FR1.1: System shall support multiple independent scouts
- FR1.2: Each scout shall implement an exploration strategy interface
- FR1.3: Scouts shall collect trajectories (state, action, reward, next_state, done)
- FR1.4: Built-in scouts: epsilon-greedy, curiosity-driven, count-based

#### FR2: Experience Aggregation

- FR2.1: System shall aggregate experiences from all scouts
- FR2.2: Aggregation strategies: simple pooling, weighted by reward, weighted by novelty
- FR2.3: System shall support filtering experiences based on criteria

#### FR3: Shared Learner

- FR3.1: Single learner shall update from aggregated experience
- FR3.2: Support standard RL algorithms (at minimum: DQN, PPO)
- FR3.3: Updated policy shall optionally propagate to scouts

#### FR4: Execution Modes

- FR4.1: Sequential execution mode (scouts run one at a time)
- FR4.2: Parallel execution mode (scouts run concurrently)
- FR4.3: Results shall be comparable between modes

#### FR5: Environments

- FR5.1: Support for sparse-reward grid worlds
- FR5.2: Support for custom environments via interface
- FR5.3: Minimal environments included for demos

#### FR6: Visualization

- FR6.1: Visualize exploration coverage across scouts
- FR6.2: Show learning curves comparing single vs. many-eyes
- FR6.3: Reconstruct "many eyes" exploration as animation

#### FR7: Reproducibility

- FR7.1: Experiments shall be reproducible with fixed seeds
- FR7.2: Configuration files for all experiment parameters
- FR7.3: Scripts to regenerate all figures/results

### Non-Functional Requirements

#### NFR1: Simplicity

- Code shall prioritize readability over performance
- Single-file implementations preferred where practical
- Minimal external dependencies

#### NFR2: Portability

- Run on laptop with no GPU required
- Python 3.8+ with standard scientific stack
- No cloud services required

#### NFR3: Documentation

- All public functions documented
- README with quick start guide
- Architecture documentation
- Experiment reproduction instructions

## Success Criteria

1. **Demo**: Show improved learning in sparse grid world with many-eyes vs. single explorer
2. **Clarity**: A graduate student can understand the codebase in one sitting
3. **Reproducibility**: Anyone can reproduce experiments with provided scripts

## Milestones

### M1: Foundation (MVP) - COMPLETE

- [x] Basic scout interface (RandomScout, EpsilonGreedyScout)
- [x] Simple experience buffer (ReplayBuffer)
- [x] DQN learner with target network
- [x] SparseGridWorld environment
- [x] Sequential execution
- [x] 42 tests passing

### M2: Diversity - COMPLETE

- [x] Multiple scout strategies (Random, EpsilonGreedy, Curious, Optimistic)
- [x] Heterogeneous scout configurations
- [x] Simple aggregation
- [x] CLI visualization with policy arrows
- [x] Result plots with matplotlib
- [x] Reproducible experiments (5 seeds)
- [x] Web UI demo (FastAPI + Yew/WASM)
- [ ] Weighted aggregation strategies (moved to backlog)

### M3: Parallel - NOT STARTED

- [ ] Parallel execution mode
- [ ] Performance comparison tools
- [ ] Extended experiments

### M4: Polish - PARTIAL

- [x] ELI5 documentation
- [x] Results documentation
- [x] Architecture documentation
- [ ] Complete API documentation
- [ ] Tutorial notebook
- [ ] Example notebooks

## Constraints

### Technical Constraints

- Python for accessibility and ecosystem
- NumPy/PyTorch for computation
- Matplotlib for visualization
- No proprietary dependencies

### Resource Constraints

- Development on single developer workstation
- No cloud compute budget
- No dedicated hardware

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Sparse rewards too sparse for demos | Medium | High | Tune environment difficulty |
| Parallel mode introduces non-determinism | Medium | Medium | Extensive testing, clear documentation |
| Scope creep into optimization | High | Medium | Strict adherence to non-goals |
| Results don't show clear benefit | Low | High | Start with favorable environments |

## Open Questions (Answered)

1. **Which exploration strategies to include in initial release?**
   - Answer: RandomScout, EpsilonGreedyScout, CuriousScout, OptimisticScout (4 strategies implemented)

2. **How to handle continuous action spaces (if at all)?**
   - Answer: Deferred to backlog. Current focus is discrete actions for clarity.

3. **What's the right balance of environments (toy vs. realistic)?**
   - Answer: Start with toy (SparseGridWorld). Demonstrates concept clearly. More environments in backlog.

## Remaining Open Questions

1. ~~Should we add a web UI (Rust/Yew) or stick with CLI?~~ **Answered**: Web UI implemented with FastAPI backend + Yew/WASM frontend
2. Is weighted aggregation worth implementing given diversity experiment results?
3. What content format is most valuable: YouTube, blog, or notebook?

## References

- Research on structured exploration in sparse-reward environments
- Multi-policy exploration (scout-based methods)
- Intrinsic motivation and information-driven learning

## Appendix: Terminology

- **Scout**: An exploratory agent that gathers experience
- **Many-eyes**: The approach of using multiple scouts for diverse exploration
- **Sparse reward**: Environment where non-zero rewards are rare
- **Aggregation**: Combining experiences from multiple scouts
- **Shared learner**: Single policy/value network updated from aggregated experience
