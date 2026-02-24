//! Event types matching the backend WebSocket protocol.

use serde::{Deserialize, Serialize};

/// Data for a single scout move in batch events
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Fields are part of the WebSocket protocol
pub struct ScoutMoveData {
    pub scout_id: String,
    pub scout_index: usize,
    pub position: (i32, i32),
    pub action: i32,
    pub reward: f64,
    pub done: bool,
}

/// Batch scout moves event (all scouts move simultaneously)
#[derive(Debug, Clone, Deserialize)]
pub struct BatchScoutMovesEvent {
    pub moves: Vec<ScoutMoveData>,
    pub step: i32,
}

/// Server -> Client events
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ServerEvent {
    #[serde(rename = "scout_move")]
    ScoutMove(ScoutMoveEvent),
    #[serde(rename = "batch_scout_moves")]
    BatchScoutMoves(BatchScoutMovesEvent),
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
#[allow(dead_code)] // Fields are part of the WebSocket protocol
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
#[allow(dead_code)] // Fields are part of the WebSocket protocol
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
    pub average_steps: f64,
    pub episode_reward: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PolicyUpdateEvent {
    pub policy: Vec<Vec<i32>>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Fields are part of the WebSocket protocol
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
    pub average_steps: Vec<f64>,
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

/// Recorded event for replay
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedEvent {
    pub index: usize,
    pub event: RecordedEventType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordedEventType {
    ScoutMove { scout_index: usize, position: (i32, i32) },
    BatchScoutMoves { moves: Vec<(usize, (i32, i32))> }, // (scout_index, position)
    EpisodeComplete { scout_index: usize, reached_goal: bool, total_reward: f64 },
    TrainingUpdate { episode: i32, total_episodes: i32, success_rate: f64 },
    PolicyUpdate { policy: Vec<Vec<i32>> },
}

/// Application state
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppState {
    pub connected: bool,
    pub training: bool,
    pub paused: bool,
    pub grid_size: i32,
    pub n_scouts: i32,
    pub scouts: Vec<ScoutInfo>,
    pub policy: Vec<Vec<i32>>,
    pub policy_received: bool,  // True after first PolicyUpdate
    pub visited_cells: Vec<Vec<f64>>,  // Combined visit counts per cell
    pub per_scout_visited_cells: Vec<Vec<Vec<f64>>>,  // Per-scout visit counts [scout][row][col]
    pub current_episode: i32,
    pub total_episodes: i32,
    pub success_rate: f64,
    pub average_steps: f64,
    pub history: TrainingHistory,
    pub error_message: Option<String>,

    // Replay state
    pub recorded_events: Vec<RecordedEvent>,
    pub replay_mode: bool,
    pub replay_index: usize,
    pub replay_playing: bool,
    pub replay_speed: f64,  // 0.1 to 2.0
}

impl AppState {
    pub fn new(grid_size: i32) -> Self {
        Self::new_with_scouts(grid_size, 5)
    }

    pub fn new_with_scouts(grid_size: i32, n_scouts: i32) -> Self {
        let size = grid_size as usize;
        let n = n_scouts as usize;
        Self {
            grid_size,
            n_scouts,
            visited_cells: vec![vec![0.0; size]; size],
            per_scout_visited_cells: vec![vec![vec![0.0; size]; size]; n],
            policy: vec![vec![0; size]; size],
            ..Default::default()
        }
    }

    #[allow(dead_code)] // Available for future use
    pub fn reset_for_training(&mut self, n_scouts: i32) {
        let size = self.grid_size as usize;
        let n = n_scouts as usize;
        self.n_scouts = n_scouts;
        self.scouts = (0..n)
            .map(|i| ScoutInfo {
                index: i,
                position: (0, 0),
                ..Default::default()
            })
            .collect();
        self.visited_cells = vec![vec![0.0; size]; size];
        self.per_scout_visited_cells = vec![vec![vec![0.0; size]; size]; n];
        self.policy = vec![vec![0; size]; size];
        self.current_episode = 0;
        self.success_rate = 0.0;
        self.average_steps = 0.0;
        self.history = TrainingHistory::default();
        self.error_message = None;
        self.training = true;
        self.paused = false;
    }
}
