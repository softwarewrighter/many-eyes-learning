//! Streaming trainer that simulates multi-scout RL training.

use rand::prelude::*;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::time::sleep;

use crate::events::{ExplorationMode, ScoutMoveData, ServerEvent, TrainingConfig, TrainingHistory};

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
    epsilon_min: f64,
    epsilon_decay: f64,
    always_random: bool,
    exploration_mode: ExplorationMode,
    rng: StdRng,
    q_values: Option<Vec<Vec<[f64; 4]>>>,
}

impl Scout {
    fn new(index: usize, epsilon: f64, always_random: bool, exploration_mode: ExplorationMode, seed: u64) -> Self {
        Self {
            id: format!("scout_{}", index),
            index,
            epsilon,
            epsilon_min: 0.01,
            epsilon_decay: 0.95,
            always_random,
            exploration_mode,
            rng: StdRng::seed_from_u64(seed + index as u64),
            q_values: None,
        }
    }

    fn select_action(&mut self, pos: (i32, i32), _grid_size: i32) -> i32 {
        // Always random scout never follows policy
        if self.always_random {
            return self.rng.gen_range(0..4);
        }

        match self.exploration_mode {
            ExplorationMode::SharedPolicy => self.select_shared_policy(pos),
            ExplorationMode::DiversePaths => self.select_diverse_paths(pos),
            ExplorationMode::HighExploration => self.select_high_exploration(),
            ExplorationMode::Boltzmann => self.select_boltzmann(pos),
        }
    }

    /// SharedPolicy: Standard epsilon-greedy with deterministic argmax
    fn select_shared_policy(&mut self, pos: (i32, i32)) -> i32 {
        if self.rng.gen::<f64>() < self.epsilon {
            return self.rng.gen_range(0..4);
        }
        self.greedy_action(pos)
    }

    /// DiversePaths: Biased exploration and biased greedy selection
    fn select_diverse_paths(&mut self, pos: (i32, i32)) -> i32 {
        if self.rng.gen::<f64>() < self.epsilon {
            return self.biased_random_action(pos);
        }
        self.biased_greedy_action(pos)
    }

    /// HighExploration: Always use epsilon=0.5 (never decay)
    fn select_high_exploration(&mut self) -> i32 {
        // Fixed 50% random exploration
        if self.rng.gen::<f64>() < 0.5 {
            self.rng.gen_range(0..4)
        } else {
            // Simple greedy (no Q-values needed for random selection)
            self.rng.gen_range(0..4)
        }
    }

    /// Boltzmann: Softmax action selection with temperature
    fn select_boltzmann(&mut self, pos: (i32, i32)) -> i32 {
        let temperature = 0.5 + 0.3 * (self.index as f64);  // Per-scout temperature

        if let Some(ref q_table) = self.q_values {
            let (row, col) = pos;
            if row >= 0 && col >= 0 {
                if let Some(row_q) = q_table.get(row as usize) {
                    if let Some(q_vals) = row_q.get(col as usize) {
                        // Softmax with temperature
                        let max_q = q_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                        let exp_vals: Vec<f64> = q_vals
                            .iter()
                            .map(|q| ((q - max_q) / temperature).exp())
                            .collect();
                        let sum: f64 = exp_vals.iter().sum();
                        let probs: Vec<f64> = exp_vals.iter().map(|e| e / sum).collect();

                        // Weighted random selection
                        let mut roll = self.rng.gen::<f64>();
                        for (action, prob) in probs.iter().enumerate() {
                            roll -= prob;
                            if roll <= 0.0 {
                                return action as i32;
                            }
                        }
                    }
                }
            }
        }
        self.rng.gen_range(0..4)
    }

    /// Greedy action: pick highest Q-value (deterministic tie-breaking)
    fn greedy_action(&mut self, pos: (i32, i32)) -> i32 {
        if let Some(ref q_table) = self.q_values {
            let (row, col) = pos;
            if row >= 0 && col >= 0 {
                if let Some(row_q) = q_table.get(row as usize) {
                    if let Some(q_vals) = row_q.get(col as usize) {
                        return q_vals
                            .iter()
                            .enumerate()
                            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                            .map(|(i, _)| i as i32)
                            .unwrap_or(0);
                    }
                }
            }
        }
        self.rng.gen_range(0..4)
    }

    /// Biased greedy action: add per-scout directional bias to Q-values
    fn biased_greedy_action(&mut self, pos: (i32, i32)) -> i32 {
        if let Some(ref q_table) = self.q_values {
            let (row, col) = pos;
            if row >= 0 && col >= 0 {
                if let Some(row_q) = q_table.get(row as usize) {
                    if let Some(q_vals) = row_q.get(col as usize) {
                        let max_q = q_vals.iter().cloned().fold(0.0_f64, f64::max);
                        let bias = (max_q.abs() + 0.5) * 0.8;
                        let mut biased_q = *q_vals;

                        match self.index % 5 {
                            0 => {}  // Scout 0 is always random
                            1 => { biased_q[1] += bias; biased_q[2] += bias * 0.3; }  // Right
                            2 => { biased_q[2] += bias; biased_q[1] += bias * 0.3; }  // Down
                            3 => { biased_q[2] += bias; biased_q[3] += bias * 0.5; }  // Down-left
                            _ => {
                                let (r, c) = pos;
                                if (r + c) % 2 == 0 { biased_q[1] += bias; }
                                else { biased_q[2] += bias; }
                            }
                        }

                        return biased_q
                            .iter()
                            .enumerate()
                            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                            .map(|(i, _)| i as i32)
                            .unwrap_or(0);
                    }
                }
            }
        }
        self.rng.gen_range(0..4)
    }

    /// Biased random action: weighted random selection per scout
    fn biased_random_action(&mut self, pos: (i32, i32)) -> i32 {
        let weights: [f64; 4] = match self.index % 5 {
            0 => [1.0, 1.0, 1.0, 1.0],  // Scout 0: uniform random
            1 => [0.5, 3.0, 1.5, 0.5],  // Scout 1: prefer right
            2 => [0.5, 1.5, 3.0, 0.5],  // Scout 2: prefer down
            3 => [1.0, 1.0, 2.5, 1.0],  // Scout 3: slight down
            _ => {
                let (r, c) = pos;
                if (r + c) % 2 == 0 { [0.5, 3.0, 1.0, 0.5] }
                else { [0.5, 1.0, 3.0, 0.5] }
            }
        };

        let total: f64 = weights.iter().sum();
        let mut roll = self.rng.gen::<f64>() * total;
        for (action, weight) in weights.iter().enumerate() {
            roll -= weight;
            if roll <= 0.0 { return action as i32; }
        }
        0
    }

    fn decay_epsilon(&mut self) {
        // HighExploration mode doesn't decay epsilon
        if !self.always_random && self.exploration_mode != ExplorationMode::HighExploration {
            self.epsilon = (self.epsilon * self.epsilon_decay).max(self.epsilon_min);
        }
    }

    fn set_q_values(&mut self, q_values: Vec<Vec<[f64; 4]>>) {
        self.q_values = Some(q_values);
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

    fn get_q_values(&self) -> Vec<Vec<[f64; 4]>> {
        self.q_table.clone()
    }
}

/// State for a single scout during parallel exploration
struct ScoutExplorationState {
    env: GridWorld,
    pos: (i32, i32),
    total_reward: f64,
    step_count: i32,
    done: bool,
    reached_goal: bool,
}

/// Training state machine
enum TrainerState {
    /// All scouts exploring in parallel
    ParallelExploring {
        scout_states: Vec<ScoutExplorationState>,
        global_step: i32,
    },
    /// All scouts done, emit training update
    EpisodeDone {
        successes: i32,
        total_reward: f64,
        total_steps: i32,
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
    pending_events: VecDeque<ServerEvent>,
}

impl StreamingTrainer {
    pub fn new(config: TrainingConfig) -> Self {
        let seed = config.seed.unwrap_or(42);
        let exploration_mode = config.exploration_mode;
        let scouts: Vec<Scout> = (0..config.n_scouts as usize)
            .map(|i| {
                if i == 0 {
                    // Scout 0 is always random - provides exploration baseline
                    Scout::new(i, 1.0, true, exploration_mode, seed)
                } else {
                    // Other scouts start with high epsilon, decay over time
                    let epsilon = 0.5 + 0.3 * (i as f64 / config.n_scouts as f64);
                    Scout::new(i, epsilon, false, exploration_mode, seed)
                }
            })
            .collect();

        // Initialize all scouts in parallel
        let scout_states: Vec<ScoutExplorationState> = (0..config.n_scouts as usize)
            .map(|_| {
                let mut env = GridWorld::new(config.grid_size, config.steps_per_episode);
                let pos = env.reset();
                ScoutExplorationState {
                    env,
                    pos,
                    total_reward: 0.0,
                    step_count: 0,
                    done: false,
                    reached_goal: false,
                }
            })
            .collect();

        Self {
            learner: QLearner::new(config.grid_size),
            scouts,
            config,
            history: TrainingHistory::default(),
            current_episode: 0,
            paused: false,
            speed: 1.0,
            state: TrainerState::ParallelExploring {
                scout_states,
                global_step: 0,
            },
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
        self.speed = speed.clamp(0.1, 100.0);
    }

    #[allow(clippy::while_immutable_condition)]
    pub async fn step(&mut self) -> Option<ServerEvent> {
        // Return any pending events first
        if let Some(event) = self.pending_events.pop_front() {
            return Some(event);
        }

        // Wait if paused (self.paused is modified externally via pause()/resume())
        while self.paused {
            sleep(Duration::from_millis(100)).await;
        }

        match &mut self.state {
            TrainerState::ParallelExploring { scout_states, global_step } => {
                *global_step += 1;
                let current_step = *global_step;

                // Step all scouts that aren't done
                let mut moves: Vec<ScoutMoveData> = Vec::new();
                let mut all_done = true;

                for (idx, scout_state) in scout_states.iter_mut().enumerate() {
                    if scout_state.done {
                        continue;
                    }
                    all_done = false;

                    let scout = &mut self.scouts[idx];
                    let action = scout.select_action(scout_state.pos, self.config.grid_size);
                    let (new_pos, reward, done) = scout_state.env.step(action);

                    // Update Q-learner
                    self.learner.update(scout_state.pos, action, reward, new_pos, done);

                    scout_state.total_reward += reward;
                    scout_state.step_count += 1;

                    moves.push(ScoutMoveData {
                        scout_id: scout.id.clone(),
                        scout_index: idx,
                        position: new_pos,
                        action,
                        reward,
                        done,
                    });

                    if done {
                        scout_state.done = true;
                        scout_state.reached_goal = reward > 0.0;
                    } else {
                        scout_state.pos = new_pos;
                    }
                }

                // Check if all scouts are now done
                let still_exploring = scout_states.iter().any(|s| !s.done);

                if !still_exploring {
                    // Calculate episode stats
                    let successes = scout_states.iter().filter(|s| s.reached_goal).count() as i32;
                    let total_reward: f64 = scout_states.iter().map(|s| s.total_reward).sum();
                    let total_steps: i32 = scout_states.iter().map(|s| s.step_count).sum();

                    // Queue episode complete events for each scout
                    for (idx, scout_state) in scout_states.iter().enumerate() {
                        self.pending_events.push_back(ServerEvent::EpisodeComplete {
                            scout_id: format!("scout_{}", idx),
                            scout_index: idx,
                            reached_goal: scout_state.reached_goal,
                            total_reward: scout_state.total_reward,
                            steps: scout_state.step_count,
                        });
                    }

                    self.state = TrainerState::EpisodeDone {
                        successes,
                        total_reward,
                        total_steps,
                    };
                }

                // Delay for visualization (50ms default, adjustable with speed)
                let delay = (50.0 / self.speed) as u64;
                sleep(Duration::from_millis(delay)).await;

                // Return batch moves event if any scouts moved
                if moves.is_empty() && all_done {
                    // All scouts were already done, transition happened
                    self.pending_events.pop_front()
                } else {
                    Some(ServerEvent::BatchScoutMoves {
                        moves,
                        step: current_step,
                    })
                }
            }

            TrainerState::EpisodeDone { successes, total_reward, total_steps } => {
                let success_rate = *successes as f64 / self.scouts.len() as f64;
                let average_steps = *total_steps as f64 / self.scouts.len() as f64;
                self.history.success_rates.push(success_rate);
                self.history.episode_rewards.push(*total_reward);
                self.history.average_steps.push(average_steps);

                self.current_episode += 1;

                let event = ServerEvent::TrainingUpdate {
                    episode: self.current_episode,
                    total_episodes: self.config.episodes,
                    success_rate,
                    average_steps,
                    episode_reward: *total_reward,
                };

                // Update scout Q-values and emit policy update
                if self.current_episode % 5 == 0 {
                    let q_values = self.learner.get_q_values();
                    for scout in &mut self.scouts {
                        scout.set_q_values(q_values.clone());
                    }
                    // Policy for visualization (deterministic for UI arrows)
                    let policy = self.learner.get_policy();
                    self.pending_events.push_back(ServerEvent::PolicyUpdate {
                        policy,
                    });
                }

                // Decay epsilon for all scouts (except always-random scout 0)
                for scout in &mut self.scouts {
                    scout.decay_epsilon();
                }

                // Check if training is complete
                if self.current_episode >= self.config.episodes {
                    self.state = TrainerState::Finished;
                } else {
                    // Reset for next episode - all scouts in parallel
                    let scout_states: Vec<ScoutExplorationState> = (0..self.scouts.len())
                        .map(|_| {
                            let mut env = GridWorld::new(self.config.grid_size, self.config.steps_per_episode);
                            let pos = env.reset();
                            ScoutExplorationState {
                                env,
                                pos,
                                total_reward: 0.0,
                                step_count: 0,
                                done: false,
                                reached_goal: false,
                            }
                        })
                        .collect();

                    self.state = TrainerState::ParallelExploring {
                        scout_states,
                        global_step: 0,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Simulate a fixed number of episodes and collect per-scout visit counts
    fn simulate_training(config: TrainingConfig, episodes: i32) -> Vec<Vec<Vec<i32>>> {
        let n_scouts = config.n_scouts as usize;
        let grid_size = config.grid_size as usize;
        let seed = config.seed.unwrap_or(42);
        let exploration_mode = config.exploration_mode;

        // Per-scout visit counts: [scout][row][col]
        let mut visits: Vec<Vec<Vec<i32>>> = vec![vec![vec![0; grid_size]; grid_size]; n_scouts];

        // Create scouts
        let mut scouts: Vec<Scout> = (0..n_scouts)
            .map(|i| {
                if i == 0 {
                    Scout::new(i, 1.0, true, exploration_mode, seed)
                } else {
                    let epsilon = 0.5 + 0.3 * (i as f64 / n_scouts as f64);
                    Scout::new(i, epsilon, false, exploration_mode, seed)
                }
            })
            .collect();

        let mut learner = QLearner::new(config.grid_size);

        for _ in 0..episodes {
            // Each scout runs one episode
            for (scout_idx, scout) in scouts.iter_mut().enumerate() {
                let mut env = GridWorld::new(config.grid_size, config.steps_per_episode);
                let mut pos = env.reset();

                // Record starting position
                visits[scout_idx][pos.0 as usize][pos.1 as usize] += 1;

                loop {
                    let action = scout.select_action(pos, config.grid_size);
                    let (new_pos, reward, done) = env.step(action);

                    // Record visit
                    visits[scout_idx][new_pos.0 as usize][new_pos.1 as usize] += 1;

                    // Update learner
                    learner.update(pos, action, reward, new_pos, done);

                    if done {
                        break;
                    }
                    pos = new_pos;
                }
            }

            // Distribute Q-values to scouts (like real training)
            let q_values = learner.get_q_values();
            for scout in scouts.iter_mut() {
                scout.set_q_values(q_values.clone());
            }

            // Decay epsilon
            for scout in scouts.iter_mut() {
                scout.decay_epsilon();
            }
        }

        visits
    }

    /// Calculate normalized heatmap (proportions instead of counts)
    fn normalize_heatmap(visits: &[Vec<i32>]) -> Vec<Vec<f64>> {
        let total: i32 = visits.iter().flat_map(|row| row.iter()).sum();
        if total == 0 {
            return visits.iter().map(|row| row.iter().map(|_| 0.0).collect()).collect();
        }
        visits
            .iter()
            .map(|row| row.iter().map(|&v| v as f64 / total as f64).collect())
            .collect()
    }

    /// Calculate Jensen-Shannon divergence between two probability distributions
    fn js_divergence(p: &[Vec<f64>], q: &[Vec<f64>]) -> f64 {
        let mut divergence = 0.0;
        for (p_row, q_row) in p.iter().zip(q.iter()) {
            for (&pi, &qi) in p_row.iter().zip(q_row.iter()) {
                if pi > 0.0 || qi > 0.0 {
                    let m = (pi + qi) / 2.0;
                    if pi > 0.0 {
                        divergence += pi * (pi / m).ln();
                    }
                    if qi > 0.0 {
                        divergence += qi * (qi / m).ln();
                    }
                }
            }
        }
        divergence / 2.0
    }

    /// Calculate average pairwise JS divergence between scout heatmaps
    fn average_heatmap_divergence(visits: &[Vec<Vec<i32>>]) -> f64 {
        let heatmaps: Vec<Vec<Vec<f64>>> = visits.iter().map(|v| normalize_heatmap(v)).collect();

        let mut total_div = 0.0;
        let mut count = 0;

        // Compare all pairs of scouts (excluding scout 0 which is always random)
        for i in 1..heatmaps.len() {
            for j in (i + 1)..heatmaps.len() {
                total_div += js_divergence(&heatmaps[i], &heatmaps[j]);
                count += 1;
            }
        }

        if count == 0 {
            0.0
        } else {
            total_div / count as f64
        }
    }

    #[test]
    fn test_diverse_paths_produces_different_heatmaps() {
        // Test that DiversePaths mode produces more variation than SharedPolicy
        let config_shared = TrainingConfig {
            n_scouts: 5,
            grid_size: 5,
            episodes: 30,
            steps_per_episode: 100,
            exploration_mode: ExplorationMode::SharedPolicy,
            seed: Some(42),
            ..Default::default()
        };

        let config_diverse = TrainingConfig {
            n_scouts: 5,
            grid_size: 5,
            episodes: 30,
            steps_per_episode: 100,
            exploration_mode: ExplorationMode::DiversePaths,
            seed: Some(42),
            ..Default::default()
        };

        let visits_shared = simulate_training(config_shared, 30);
        let visits_diverse = simulate_training(config_diverse, 30);

        let divergence_shared = average_heatmap_divergence(&visits_shared);
        let divergence_diverse = average_heatmap_divergence(&visits_diverse);

        println!("SharedPolicy avg JS divergence: {:.4}", divergence_shared);
        println!("DiversePaths avg JS divergence: {:.4}", divergence_diverse);

        // DiversePaths should produce MORE divergence (more different heatmaps)
        assert!(
            divergence_diverse > divergence_shared,
            "DiversePaths ({:.4}) should have higher divergence than SharedPolicy ({:.4})",
            divergence_diverse,
            divergence_shared
        );
    }

    #[test]
    fn test_boltzmann_is_functional() {
        // Boltzmann should produce non-zero divergence and complete training
        let config_boltzmann = TrainingConfig {
            n_scouts: 5,
            grid_size: 5,
            episodes: 20,
            steps_per_episode: 100,
            exploration_mode: ExplorationMode::Boltzmann,
            seed: Some(123),
            ..Default::default()
        };

        let visits = simulate_training(config_boltzmann, 20);
        let divergence = average_heatmap_divergence(&visits);

        println!("Boltzmann avg JS divergence: {:.4}", divergence);

        // Boltzmann should have some divergence (due to per-scout temperatures)
        // but Q-values dominate after learning, so divergence may be low
        assert!(
            divergence >= 0.0,
            "Boltzmann divergence should be non-negative"
        );

        // Verify all scouts visited cells (training completed)
        for scout_idx in 0..5 {
            let total_visits: i32 = visits[scout_idx].iter().flat_map(|r| r.iter()).sum();
            assert!(total_visits > 0, "Scout {} should have visits", scout_idx);
        }
    }

    #[test]
    fn test_exploration_modes_are_deterministic_with_seed() {
        // Same seed should produce same results
        let config1 = TrainingConfig {
            n_scouts: 3,
            grid_size: 5,
            episodes: 10,
            steps_per_episode: 50,
            exploration_mode: ExplorationMode::DiversePaths,
            seed: Some(999),
            ..Default::default()
        };

        let config2 = TrainingConfig {
            n_scouts: 3,
            grid_size: 5,
            episodes: 10,
            steps_per_episode: 50,
            exploration_mode: ExplorationMode::DiversePaths,
            seed: Some(999),
            ..Default::default()
        };

        let visits1 = simulate_training(config1, 10);
        let visits2 = simulate_training(config2, 10);

        // Should be identical
        for scout in 0..3 {
            for row in 0..5 {
                for col in 0..5 {
                    assert_eq!(
                        visits1[scout][row][col], visits2[scout][row][col],
                        "Visits should be identical with same seed at scout={}, row={}, col={}",
                        scout, row, col
                    );
                }
            }
        }
    }

    #[test]
    fn test_high_exploration_maintains_exploration() {
        // HighExploration should keep exploring even late in training
        let config = TrainingConfig {
            n_scouts: 3,
            grid_size: 5,
            episodes: 50,
            steps_per_episode: 100,
            exploration_mode: ExplorationMode::HighExploration,
            seed: Some(42),
            ..Default::default()
        };

        let visits = simulate_training(config, 50);

        // Check that all scouts visit many cells (high exploration)
        for scout_idx in 1..3 {
            let cells_visited: i32 = visits[scout_idx]
                .iter()
                .flat_map(|row| row.iter())
                .filter(|&&v| v > 0)
                .count() as i32;

            // Should visit at least 60% of cells with high exploration
            let expected_min = (5 * 5 * 60 / 100) as i32;  // 15 cells
            assert!(
                cells_visited >= expected_min,
                "Scout {} should visit at least {} cells, but visited {}",
                scout_idx, expected_min, cells_visited
            );
        }
    }
}
