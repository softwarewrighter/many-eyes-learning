//! Streaming trainer that simulates multi-scout RL training.

use rand::prelude::*;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::time::sleep;

use crate::events::{ServerEvent, TrainingConfig, TrainingHistory};

/// Grid world environment
struct GridWorld {
    size: i32,
    position: (i32, i32),
    goal: (i32, i32),
    steps: i32,
    max_steps: i32,
}

impl GridWorld {
    fn new(size: i32, max_steps: i32) -> Self {
        Self {
            size,
            position: (0, 0),
            goal: (size - 1, size - 1),
            steps: 0,
            max_steps,
        }
    }

    fn reset(&mut self) -> (i32, i32) {
        self.position = (0, 0);
        self.steps = 0;
        self.position
    }

    fn step(&mut self, action: i32) -> ((i32, i32), f64, bool) {
        self.steps += 1;

        let (row, col) = self.position;
        let new_pos = match action {
            0 => ((row - 1).max(0), col),                // up
            1 => (row, (col + 1).min(self.size - 1)),    // right
            2 => ((row + 1).min(self.size - 1), col),    // down
            3 => (row, (col - 1).max(0)),                // left
            _ => (row, col),
        };

        self.position = new_pos;

        let at_goal = self.position == self.goal;
        let reward = if at_goal { 1.0 } else { -0.01 };
        let done = at_goal || self.steps >= self.max_steps;

        (self.position, reward, done)
    }
}

/// Scout with exploration strategy
struct Scout {
    id: String,
    index: usize,
    epsilon: f64,
    rng: StdRng,
    policy: Option<Vec<Vec<i32>>>,
}

impl Scout {
    fn new(index: usize, epsilon: f64, seed: u64) -> Self {
        Self {
            id: format!("scout_{}", index),
            index,
            epsilon,
            rng: StdRng::seed_from_u64(seed + index as u64),
            policy: None,
        }
    }

    fn select_action(&mut self, pos: (i32, i32), _grid_size: i32) -> i32 {
        if self.rng.gen::<f64>() < self.epsilon {
            self.rng.gen_range(0..4)
        } else if let Some(ref policy) = self.policy {
            let (row, col) = pos;
            if row >= 0 && col >= 0 {
                if let Some(row_actions) = policy.get(row as usize) {
                    if let Some(&action) = row_actions.get(col as usize) {
                        return action;
                    }
                }
            }
            self.rng.gen_range(0..4)
        } else {
            self.rng.gen_range(0..4)
        }
    }

    fn set_policy(&mut self, policy: Vec<Vec<i32>>) {
        self.policy = Some(policy);
    }
}

/// Simple Q-learning for policy updates
struct QLearner {
    q_table: Vec<Vec<[f64; 4]>>,
    learning_rate: f64,
    gamma: f64,
}

impl QLearner {
    fn new(grid_size: i32) -> Self {
        let size = grid_size as usize;
        Self {
            q_table: vec![vec![[0.0; 4]; size]; size],
            learning_rate: 0.1,
            gamma: 0.99,
        }
    }

    fn update(&mut self, state: (i32, i32), action: i32, reward: f64, next_state: (i32, i32), done: bool) {
        // Bounds check - ignore updates with invalid positions
        let size = self.q_table.len();
        if state.0 < 0 || state.1 < 0 || next_state.0 < 0 || next_state.1 < 0 {
            return;
        }
        let (r, c) = (state.0 as usize, state.1 as usize);
        let (nr, nc) = (next_state.0 as usize, next_state.1 as usize);
        if r >= size || c >= size || nr >= size || nc >= size {
            return;
        }
        let a = (action as usize).min(3);

        let max_next_q = if done {
            0.0
        } else {
            self.q_table[nr][nc].iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        };

        let target = reward + self.gamma * max_next_q;
        self.q_table[r][c][a] += self.learning_rate * (target - self.q_table[r][c][a]);
    }

    fn get_policy(&self) -> Vec<Vec<i32>> {
        self.q_table
            .iter()
            .map(|row| {
                row.iter()
                    .map(|q_values| {
                        q_values
                            .iter()
                            .enumerate()
                            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                            .map(|(i, _)| i as i32)
                            .unwrap_or(0)
                    })
                    .collect()
            })
            .collect()
    }
}

/// Training state machine
enum TrainerState {
    /// Running scout exploration
    Exploring {
        scout_idx: usize,
        env: GridWorld,
        pos: (i32, i32),
        total_reward: f64,
        step_count: i32,
    },
    /// Episode finished for a scout
    ScoutDone {
        scout_idx: usize,
        reached_goal: bool,
        total_reward: f64,
        steps: i32,
    },
    /// All scouts done, emit training update
    EpisodeDone {
        successes: i32,
        total_reward: f64,
    },
    /// Training complete
    Finished,
}

pub struct StreamingTrainer {
    config: TrainingConfig,
    scouts: Vec<Scout>,
    learner: QLearner,
    history: TrainingHistory,
    current_episode: i32,
    paused: bool,
    speed: f64,
    state: TrainerState,
    episode_successes: i32,
    episode_reward: f64,
    pending_events: VecDeque<ServerEvent>,
}

impl StreamingTrainer {
    pub fn new(config: TrainingConfig) -> Self {
        let seed = config.seed.unwrap_or(42);
        let scouts: Vec<Scout> = (0..config.n_scouts as usize)
            .map(|i| {
                let epsilon = if i == 0 {
                    1.0  // First scout is random
                } else {
                    0.1 + 0.2 * (i as f64 / config.n_scouts as f64)
                };
                Scout::new(i, epsilon, seed)
            })
            .collect();

        // Start first scout
        let mut env = GridWorld::new(config.grid_size, config.steps_per_episode);
        let pos = env.reset();

        Self {
            learner: QLearner::new(config.grid_size),
            scouts,
            config,
            history: TrainingHistory::default(),
            current_episode: 0,
            paused: false,
            speed: 1.0,
            state: TrainerState::Exploring {
                scout_idx: 0,
                env,
                pos,
                total_reward: 0.0,
                step_count: 0,
            },
            episode_successes: 0,
            episode_reward: 0.0,
            pending_events: VecDeque::new(),
        }
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed.clamp(0.1, 10.0);
    }

    pub async fn step(&mut self) -> Option<ServerEvent> {
        // Return any pending events first
        if let Some(event) = self.pending_events.pop_front() {
            return Some(event);
        }

        // Wait if paused
        while self.paused {
            sleep(Duration::from_millis(100)).await;
        }

        match &mut self.state {
            TrainerState::Exploring {
                scout_idx,
                env,
                pos,
                total_reward,
                step_count,
            } => {
                let scout = &mut self.scouts[*scout_idx];
                let action = scout.select_action(*pos, self.config.grid_size);
                let (new_pos, reward, done) = env.step(action);

                // Update Q-learner
                self.learner.update(*pos, action, reward, new_pos, done);

                *total_reward += reward;
                *step_count += 1;

                let event = ServerEvent::ScoutMove {
                    scout_id: scout.id.clone(),
                    scout_index: scout.index,
                    position: new_pos,
                    action,
                    reward,
                    done,
                    step: *step_count,
                };

                if done {
                    let reached_goal = reward > 0.0;
                    let tr = *total_reward;
                    let sc = *step_count;
                    let si = *scout_idx;

                    self.state = TrainerState::ScoutDone {
                        scout_idx: si,
                        reached_goal,
                        total_reward: tr,
                        steps: sc,
                    };
                } else {
                    *pos = new_pos;
                }

                // Delay for visualization (50ms default, adjustable with speed)
                let delay = (50.0 / self.speed) as u64;
                sleep(Duration::from_millis(delay)).await;

                Some(event)
            }

            TrainerState::ScoutDone {
                scout_idx,
                reached_goal,
                total_reward,
                steps,
            } => {
                let scout = &self.scouts[*scout_idx];
                let event = ServerEvent::EpisodeComplete {
                    scout_id: scout.id.clone(),
                    scout_index: *scout_idx,
                    reached_goal: *reached_goal,
                    total_reward: *total_reward,
                    steps: *steps,
                };

                if *reached_goal {
                    self.episode_successes += 1;
                }
                self.episode_reward += *total_reward;

                let next_scout_idx = *scout_idx + 1;

                if next_scout_idx < self.scouts.len() {
                    // Start next scout
                    let mut env = GridWorld::new(self.config.grid_size, self.config.steps_per_episode);
                    let pos = env.reset();
                    self.state = TrainerState::Exploring {
                        scout_idx: next_scout_idx,
                        env,
                        pos,
                        total_reward: 0.0,
                        step_count: 0,
                    };
                } else {
                    // All scouts done
                    self.state = TrainerState::EpisodeDone {
                        successes: self.episode_successes,
                        total_reward: self.episode_reward,
                    };
                }

                Some(event)
            }

            TrainerState::EpisodeDone { successes, total_reward } => {
                let success_rate = *successes as f64 / self.scouts.len() as f64;
                self.history.success_rates.push(success_rate);
                self.history.episode_rewards.push(*total_reward);
                self.history.losses.push(0.0);

                self.current_episode += 1;

                let event = ServerEvent::TrainingUpdate {
                    episode: self.current_episode,
                    total_episodes: self.config.episodes,
                    success_rate,
                    loss: 0.0,
                    episode_reward: *total_reward,
                };

                // Update scout policies and emit policy update
                if self.current_episode % 5 == 0 {
                    let policy = self.learner.get_policy();
                    for scout in &mut self.scouts {
                        scout.set_policy(policy.clone());
                    }
                    self.pending_events.push_back(ServerEvent::PolicyUpdate {
                        policy,
                    });
                }

                // Check if training is complete
                if self.current_episode >= self.config.episodes {
                    self.state = TrainerState::Finished;
                } else {
                    // Reset for next episode
                    self.episode_successes = 0;
                    self.episode_reward = 0.0;

                    let mut env = GridWorld::new(self.config.grid_size, self.config.steps_per_episode);
                    let pos = env.reset();
                    self.state = TrainerState::Exploring {
                        scout_idx: 0,
                        env,
                        pos,
                        total_reward: 0.0,
                        step_count: 0,
                    };
                }

                Some(event)
            }

            TrainerState::Finished => {
                let final_success_rate = if self.history.success_rates.is_empty() {
                    0.0
                } else {
                    let last_n: Vec<_> = self.history.success_rates.iter().rev().take(10).collect();
                    last_n.iter().copied().sum::<f64>() / last_n.len() as f64
                };

                Some(ServerEvent::TrainingComplete {
                    final_success_rate,
                    history: self.history.clone(),
                })
            }
        }
    }
}
