//! Event types for WebSocket communication.

use serde::{Deserialize, Serialize};

/// Client -> Server commands
#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct TrainingConfig {
    pub n_scouts: i32,
    pub grid_size: i32,
    pub episodes: i32,
    pub steps_per_episode: i32,
    #[serde(default)]
    pub with_obstacles: bool,
    pub seed: Option<u64>,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            n_scouts: 5,
            grid_size: 5,
            episodes: 100,
            steps_per_episode: 100,
            with_obstacles: false,
            seed: None,
        }
    }
}

/// Server -> Client events
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ServerEvent {
    #[serde(rename = "scout_move")]
    ScoutMove {
        scout_id: String,
        scout_index: usize,
        position: (i32, i32),
        action: i32,
        reward: f64,
        done: bool,
        step: i32,
    },

    #[serde(rename = "episode_complete")]
    EpisodeComplete {
        scout_id: String,
        scout_index: usize,
        reached_goal: bool,
        total_reward: f64,
        steps: i32,
    },

    #[serde(rename = "training_update")]
    TrainingUpdate {
        episode: i32,
        total_episodes: i32,
        success_rate: f64,
        loss: f64,
        episode_reward: f64,
    },

    #[serde(rename = "policy_update")]
    PolicyUpdate { policy: Vec<Vec<i32>> },

    #[serde(rename = "training_complete")]
    TrainingComplete {
        final_success_rate: f64,
        history: TrainingHistory,
    },

    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct TrainingHistory {
    pub episode_rewards: Vec<f64>,
    pub success_rates: Vec<f64>,
    pub losses: Vec<f64>,
}
