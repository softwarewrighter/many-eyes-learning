//! Event types for WebSocket communication.

use serde::{Deserialize, Serialize};

/// Exploration mode determines how scouts explore the environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExplorationMode {
    /// All scouts follow the same learned policy (deterministic argmax)
    #[default]
    SharedPolicy,
    /// Each scout has directional biases for diverse path exploration
    DiversePaths,
    /// All scouts maintain high exploration (epsilon stays at 0.5)
    HighExploration,
    /// Scouts use Boltzmann (softmax) action selection with temperature
    Boltzmann,
}

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
    #[allow(dead_code)] // Reserved for future obstacle support
    pub with_obstacles: bool,
    pub seed: Option<u64>,
    #[serde(default)]
    pub exploration_mode: ExplorationMode,
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
            exploration_mode: ExplorationMode::SharedPolicy,
        }
    }
}

/// Data for a single scout move (used in batch moves)
#[derive(Debug, Clone, Serialize)]
pub struct ScoutMoveData {
    pub scout_id: String,
    pub scout_index: usize,
    pub position: (i32, i32),
    pub action: i32,
    pub reward: f64,
    pub done: bool,
}

/// Server -> Client events
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ServerEvent {
    #[allow(dead_code)] // Kept for replay compatibility
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

    #[serde(rename = "batch_scout_moves")]
    BatchScoutMoves {
        moves: Vec<ScoutMoveData>,
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
        average_steps: f64,
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
    #[allow(dead_code)] // Reserved for error reporting
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct TrainingHistory {
    pub episode_rewards: Vec<f64>,
    pub success_rates: Vec<f64>,
    pub average_steps: Vec<f64>,
}
