//! Event types matching the backend WebSocket protocol.

use serde::{Deserialize, Serialize};

/// Server -> Client events
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ServerEvent {
    #[serde(rename = "scout_move")]
    ScoutMove(ScoutMoveEvent),
    #[serde(rename = "episode_complete")]
    EpisodeComplete(EpisodeCompleteEvent),
    #[serde(rename = "training_update")]
    TrainingUpdate(TrainingUpdateEvent),
    #[serde(rename = "policy_update")]
    PolicyUpdate(PolicyUpdateEvent),
    #[serde(rename = "training_complete")]
    TrainingComplete(TrainingCompleteEvent),
    #[serde(rename = "error")]
    Error(ErrorEvent),
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScoutMoveEvent {
    pub scout_id: String,
    pub scout_index: usize,
    pub position: (i32, i32),
    pub action: i32,
    pub reward: f64,
    pub done: bool,
    pub step: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EpisodeCompleteEvent {
    pub scout_id: String,
    pub scout_index: usize,
    pub reached_goal: bool,
    pub total_reward: f64,
    pub steps: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrainingUpdateEvent {
    pub episode: i32,
    pub total_episodes: i32,
    pub success_rate: f64,
    pub loss: f64,
    pub episode_reward: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PolicyUpdateEvent {
    pub policy: Vec<Vec<i32>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrainingCompleteEvent {
    pub final_success_rate: f64,
    pub history: TrainingHistory,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct TrainingHistory {
    #[serde(default)]
    pub episode_rewards: Vec<f64>,
    #[serde(default)]
    pub success_rates: Vec<f64>,
    #[serde(default)]
    pub losses: Vec<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorEvent {
    pub message: String,
}

/// Client -> Server commands
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "command")]
pub enum ClientCommand {
    #[serde(rename = "start")]
    Start { config: TrainingConfig },
    #[serde(rename = "pause")]
    Pause,
    #[serde(rename = "resume")]
    Resume,
    #[serde(rename = "set_speed")]
    SetSpeed { speed: f64 },
    #[serde(rename = "stop")]
    Stop,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct TrainingConfig {
    pub n_scouts: i32,
    pub grid_size: i32,
    pub episodes: i32,
    pub steps_per_episode: i32,
    pub with_obstacles: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
}

/// Scout info for display
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScoutInfo {
    pub id: String,
    pub index: usize,
    pub position: (i32, i32),
    pub total_reward: f64,
    pub episodes_completed: i32,
    pub successes: i32,
}

/// Application state
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppState {
    pub connected: bool,
    pub training: bool,
    pub paused: bool,
    pub grid_size: i32,
    pub scouts: Vec<ScoutInfo>,
    pub policy: Vec<Vec<i32>>,
    pub visited_cells: Vec<Vec<f64>>,  // Visit counts per cell
    pub current_episode: i32,
    pub total_episodes: i32,
    pub success_rate: f64,
    pub loss: f64,
    pub history: TrainingHistory,
    pub error_message: Option<String>,
}

impl AppState {
    pub fn new(grid_size: i32) -> Self {
        let size = grid_size as usize;
        Self {
            grid_size,
            visited_cells: vec![vec![0.0; size]; size],
            policy: vec![vec![0; size]; size],
            ..Default::default()
        }
    }

    pub fn reset_for_training(&mut self, n_scouts: i32) {
        let size = self.grid_size as usize;
        self.scouts = (0..n_scouts as usize)
            .map(|i| ScoutInfo {
                index: i,
                position: (0, 0),
                ..Default::default()
            })
            .collect();
        self.visited_cells = vec![vec![0.0; size]; size];
        self.policy = vec![vec![0; size]; size];
        self.current_episode = 0;
        self.success_rate = 0.0;
        self.loss = 0.0;
        self.history = TrainingHistory::default();
        self.error_message = None;
        self.training = true;
        self.paused = false;
    }
}
