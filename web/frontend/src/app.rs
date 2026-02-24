//! Main application component.

use std::rc::Rc;
use yew::prelude::*;

use crate::components::{Controls, GridPanel, LearningChart, Metrics, ScoutLegend, ReplayControls};
use crate::services::{use_websocket, WsState};
use crate::types::{AppState, ClientCommand, ServerEvent, ScoutInfo, TrainingHistory, RecordedEvent, RecordedEventType};

const WS_URL: &str = "ws://localhost:3200/ws/train/main";

/// Actions for state reducer
pub enum Action {
    ScoutMove { scout_index: usize, position: (i32, i32) },
    #[allow(dead_code)] // step available for future use
    BatchScoutMoves { moves: Vec<(usize, (i32, i32))>, step: i32 }, // (scout_index, position)
    EpisodeComplete { scout_index: usize, reached_goal: bool, total_reward: f64 },
    TrainingUpdate { episode: i32, total_episodes: i32, success_rate: f64, average_steps: f64, episode_reward: f64 },
    PolicyUpdate { policy: Vec<Vec<i32>> },
    TrainingComplete { final_success_rate: f64 },
    StartTraining { grid_size: i32, n_scouts: i32, episodes: i32 },
    SetPaused(bool),
    SetTraining(bool),
    SetError(Option<String>),
    // Replay actions
    EnterReplayMode,
    ExitReplayMode,
    ReplayPlay,
    ReplayPause,
    ReplayStep,
    ReplaySeek { index: usize },
    ReplaySetSpeed { speed: f64 },
}

/// Apply a recorded event to the state (used during replay)
fn apply_recorded_event(state: &mut AppState, event: &RecordedEventType) {
    match event {
        RecordedEventType::ScoutMove { scout_index, position } => {
            // Ensure scouts array is big enough
            while state.scouts.len() <= *scout_index {
                state.scouts.push(ScoutInfo {
                    index: state.scouts.len(),
                    id: format!("scout_{}", state.scouts.len()),
                    position: (0, 0),
                    ..Default::default()
                });
            }
            if let Some(scout) = state.scouts.get_mut(*scout_index) {
                scout.position = *position;
            }
            // Update visited cells
            let (row, col) = *position;
            if row >= 0 && col >= 0 {
                if let Some(cell) = state
                    .visited_cells
                    .get_mut(row as usize)
                    .and_then(|r| r.get_mut(col as usize))
                {
                    *cell += 1.0;
                }
                // Update per-scout visited cells
                if let Some(scout_cells) = state.per_scout_visited_cells.get_mut(*scout_index) {
                    if let Some(cell) = scout_cells
                        .get_mut(row as usize)
                        .and_then(|r| r.get_mut(col as usize))
                    {
                        *cell += 1.0;
                    }
                }
            }
        }
        RecordedEventType::BatchScoutMoves { moves } => {
            for (scout_index, position) in moves {
                // Ensure scouts array is big enough
                while state.scouts.len() <= *scout_index {
                    state.scouts.push(ScoutInfo {
                        index: state.scouts.len(),
                        id: format!("scout_{}", state.scouts.len()),
                        position: (0, 0),
                        ..Default::default()
                    });
                }
                if let Some(scout) = state.scouts.get_mut(*scout_index) {
                    scout.position = *position;
                }
                // Update visited cells
                let (row, col) = *position;
                if row >= 0 && col >= 0 {
                    if let Some(cell) = state
                        .visited_cells
                        .get_mut(row as usize)
                        .and_then(|r| r.get_mut(col as usize))
                    {
                        *cell += 1.0;
                    }
                    // Update per-scout visited cells
                    if let Some(scout_cells) = state.per_scout_visited_cells.get_mut(*scout_index) {
                        if let Some(cell) = scout_cells
                            .get_mut(row as usize)
                            .and_then(|r| r.get_mut(col as usize))
                        {
                            *cell += 1.0;
                        }
                    }
                }
            }
        }
        RecordedEventType::EpisodeComplete { scout_index, reached_goal, total_reward } => {
            if let Some(scout) = state.scouts.get_mut(*scout_index) {
                scout.total_reward += total_reward;
                scout.episodes_completed += 1;
                if *reached_goal {
                    scout.successes += 1;
                }
                scout.position = (0, 0);
            }
        }
        RecordedEventType::TrainingUpdate { episode, total_episodes, success_rate } => {
            state.current_episode = *episode;
            state.total_episodes = *total_episodes;
            state.success_rate = *success_rate;
            state.history.success_rates.push(*success_rate);
        }
        RecordedEventType::PolicyUpdate { policy } => {
            state.policy = policy.clone();
            state.policy_received = true;
        }
    }
}

impl Reducible for AppState {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new_state = (*self).clone();

        match action {
            Action::ScoutMove { scout_index, position } => {
                // Ensure scouts array is big enough
                while new_state.scouts.len() <= scout_index {
                    new_state.scouts.push(ScoutInfo {
                        index: new_state.scouts.len(),
                        id: format!("scout_{}", new_state.scouts.len()),
                        position: (0, 0),
                        ..Default::default()
                    });
                }

                // Update scout position
                if let Some(scout) = new_state.scouts.get_mut(scout_index) {
                    scout.position = position;
                }

                // Mark cell as visited (combined)
                let (row, col) = position;
                if row >= 0 && col >= 0 {
                    let size = new_state.grid_size as usize;
                    if new_state.visited_cells.len() < size {
                        new_state.visited_cells = vec![vec![0.0; size]; size];
                    }
                    if let Some(cell) = new_state
                        .visited_cells
                        .get_mut(row as usize)
                        .and_then(|r| r.get_mut(col as usize))
                    {
                        *cell += 1.0;
                    }
                    // Update per-scout visited cells
                    if let Some(scout_cells) = new_state.per_scout_visited_cells.get_mut(scout_index) {
                        if let Some(cell) = scout_cells
                            .get_mut(row as usize)
                            .and_then(|r| r.get_mut(col as usize))
                        {
                            *cell += 1.0;
                        }
                    }
                }

                // Record event for replay (only during live training)
                if new_state.training && !new_state.replay_mode {
                    let event_index = new_state.recorded_events.len();
                    new_state.recorded_events.push(RecordedEvent {
                        index: event_index,
                        event: RecordedEventType::ScoutMove { scout_index, position },
                    });
                }
            }
            Action::BatchScoutMoves { moves, step: _ } => {
                let size = new_state.grid_size as usize;

                // Ensure visited_cells is properly sized
                if new_state.visited_cells.len() < size {
                    new_state.visited_cells = vec![vec![0.0; size]; size];
                }

                // Ensure per_scout_visited_cells is properly sized
                let n_scouts = new_state.n_scouts as usize;
                if new_state.per_scout_visited_cells.len() < n_scouts {
                    new_state.per_scout_visited_cells = vec![vec![vec![0.0; size]; size]; n_scouts];
                }

                let mut recorded_moves = Vec::new();

                for (scout_index, position) in &moves {
                    // Ensure scouts array is big enough
                    while new_state.scouts.len() <= *scout_index {
                        new_state.scouts.push(ScoutInfo {
                            index: new_state.scouts.len(),
                            id: format!("scout_{}", new_state.scouts.len()),
                            position: (0, 0),
                            ..Default::default()
                        });
                    }

                    // Update scout position
                    if let Some(scout) = new_state.scouts.get_mut(*scout_index) {
                        scout.position = *position;
                    }

                    // Mark cell as visited (combined)
                    let (row, col) = *position;
                    if row >= 0 && col >= 0 {
                        if let Some(cell) = new_state
                            .visited_cells
                            .get_mut(row as usize)
                            .and_then(|r| r.get_mut(col as usize))
                        {
                            *cell += 1.0;
                        }
                        // Update per-scout visited cells
                        if let Some(scout_cells) = new_state.per_scout_visited_cells.get_mut(*scout_index) {
                            if let Some(cell) = scout_cells
                                .get_mut(row as usize)
                                .and_then(|r| r.get_mut(col as usize))
                            {
                                *cell += 1.0;
                            }
                        }
                    }

                    recorded_moves.push((*scout_index, *position));
                }

                // Record batch event for replay (only during live training)
                if new_state.training && !new_state.replay_mode && !recorded_moves.is_empty() {
                    let event_index = new_state.recorded_events.len();
                    new_state.recorded_events.push(RecordedEvent {
                        index: event_index,
                        event: RecordedEventType::BatchScoutMoves { moves: recorded_moves },
                    });
                }
            }
            Action::EpisodeComplete { scout_index, reached_goal, total_reward } => {
                if let Some(scout) = new_state.scouts.get_mut(scout_index) {
                    scout.total_reward += total_reward;
                    scout.episodes_completed += 1;
                    if reached_goal {
                        scout.successes += 1;
                    }
                    scout.position = (0, 0);
                }

                // Record event for replay
                if new_state.training && !new_state.replay_mode {
                    let event_index = new_state.recorded_events.len();
                    new_state.recorded_events.push(RecordedEvent {
                        index: event_index,
                        event: RecordedEventType::EpisodeComplete { scout_index, reached_goal, total_reward },
                    });
                }
            }
            Action::TrainingUpdate { episode, total_episodes, success_rate, average_steps, episode_reward } => {
                new_state.current_episode = episode;
                new_state.total_episodes = total_episodes;
                new_state.success_rate = success_rate;
                new_state.average_steps = average_steps;
                new_state.history.success_rates.push(success_rate);
                new_state.history.average_steps.push(average_steps);
                new_state.history.episode_rewards.push(episode_reward);

                // Record event for replay
                if new_state.training && !new_state.replay_mode {
                    let event_index = new_state.recorded_events.len();
                    new_state.recorded_events.push(RecordedEvent {
                        index: event_index,
                        event: RecordedEventType::TrainingUpdate { episode, total_episodes, success_rate },
                    });
                }
            }
            Action::PolicyUpdate { policy } => {
                // Record event for replay (before updating state so we capture the policy)
                if new_state.training && !new_state.replay_mode {
                    let event_index = new_state.recorded_events.len();
                    new_state.recorded_events.push(RecordedEvent {
                        index: event_index,
                        event: RecordedEventType::PolicyUpdate { policy: policy.clone() },
                    });
                }

                new_state.policy = policy;
                new_state.policy_received = true;
            }
            Action::TrainingComplete { final_success_rate } => {
                new_state.training = false;
                new_state.success_rate = final_success_rate;
            }
            Action::StartTraining { grid_size, n_scouts, episodes } => {
                new_state.grid_size = grid_size;
                new_state.n_scouts = n_scouts;
                new_state.total_episodes = episodes;
                new_state.scouts = (0..n_scouts as usize)
                    .map(|i| ScoutInfo {
                        index: i,
                        id: format!("scout_{}", i),
                        position: (0, 0),
                        ..Default::default()
                    })
                    .collect();
                let size = grid_size as usize;
                let n = n_scouts as usize;
                new_state.visited_cells = vec![vec![0.0; size]; size];
                new_state.per_scout_visited_cells = vec![vec![vec![0.0; size]; size]; n];
                new_state.policy = vec![vec![0; size]; size];
                new_state.policy_received = false;
                new_state.current_episode = 0;
                new_state.success_rate = 0.0;
                new_state.average_steps = 0.0;
                new_state.history = TrainingHistory::default();
                new_state.error_message = None;
                new_state.training = true;
                new_state.paused = false;
                // Clear previous recording and exit replay mode
                new_state.recorded_events = Vec::new();
                new_state.replay_mode = false;
                new_state.replay_playing = false;
                new_state.replay_index = 0;
            }
            Action::SetPaused(paused) => {
                new_state.paused = paused;
            }
            Action::SetTraining(training) => {
                new_state.training = training;
            }
            Action::SetError(error) => {
                new_state.error_message = error;
            }
            Action::EnterReplayMode => {
                if !new_state.recorded_events.is_empty() {
                    new_state.replay_mode = true;
                    new_state.replay_index = 0;
                    new_state.replay_playing = false;
                    new_state.replay_speed = 1.0;
                    // Reset visual state to beginning
                    let size = new_state.grid_size as usize;
                    let n = new_state.n_scouts as usize;
                    new_state.visited_cells = vec![vec![0.0; size]; size];
                    new_state.per_scout_visited_cells = vec![vec![vec![0.0; size]; size]; n];
                    new_state.policy = vec![vec![0; size]; size];
                    new_state.policy_received = false;
                    for scout in &mut new_state.scouts {
                        scout.position = (0, 0);
                    }
                    new_state.current_episode = 0;
                    new_state.success_rate = 0.0;
                    log::info!("Entered replay mode with {} events", new_state.recorded_events.len());
                }
            }
            Action::ExitReplayMode => {
                new_state.replay_mode = false;
                new_state.replay_playing = false;
                log::info!("Exited replay mode");
            }
            Action::ReplayPlay => {
                new_state.replay_playing = true;
            }
            Action::ReplayPause => {
                new_state.replay_playing = false;
            }
            Action::ReplayStep => {
                // Apply the next event
                if new_state.replay_index < new_state.recorded_events.len() {
                    let event = new_state.recorded_events[new_state.replay_index].event.clone();
                    apply_recorded_event(&mut new_state, &event);
                    new_state.replay_index += 1;

                    // Auto-pause when reaching the end
                    if new_state.replay_index >= new_state.recorded_events.len() {
                        new_state.replay_playing = false;
                    }
                }
            }
            Action::ReplaySeek { index } => {
                // Reset and replay up to the given index
                let size = new_state.grid_size as usize;
                let n = new_state.n_scouts as usize;
                new_state.visited_cells = vec![vec![0.0; size]; size];
                new_state.per_scout_visited_cells = vec![vec![vec![0.0; size]; size]; n];
                new_state.policy = vec![vec![0; size]; size];
                new_state.policy_received = false;
                for scout in &mut new_state.scouts {
                    scout.position = (0, 0);
                    scout.total_reward = 0.0;
                    scout.episodes_completed = 0;
                    scout.successes = 0;
                }
                new_state.current_episode = 0;
                new_state.success_rate = 0.0;
                new_state.history = TrainingHistory::default();

                // Replay events up to index
                let target = index.min(new_state.recorded_events.len());
                for i in 0..target {
                    let event = new_state.recorded_events[i].event.clone();
                    apply_recorded_event(&mut new_state, &event);
                }
                new_state.replay_index = target;
            }
            Action::ReplaySetSpeed { speed } => {
                new_state.replay_speed = speed.clamp(0.1, 10.0);
            }
        }

        Rc::new(new_state)
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let state = use_reducer(|| AppState::new(5));
    let ws_state = use_state(|| WsState::Disconnected);
    let ws = use_websocket();

    // Handle server events
    let on_event = {
        let state = state.dispatcher();
        Callback::from(move |event: ServerEvent| {
            match event {
                ServerEvent::ScoutMove(e) => {
                    state.dispatch(Action::ScoutMove {
                        scout_index: e.scout_index,
                        position: e.position,
                    });
                }
                ServerEvent::BatchScoutMoves(e) => {
                    // Convert to simpler format for the action
                    let moves: Vec<(usize, (i32, i32))> = e.moves
                        .iter()
                        .map(|m| (m.scout_index, m.position))
                        .collect();
                    state.dispatch(Action::BatchScoutMoves {
                        moves,
                        step: e.step,
                    });
                }
                ServerEvent::EpisodeComplete(e) => {
                    log::info!("EpisodeComplete: scout={}, goal={}", e.scout_index, e.reached_goal);
                    state.dispatch(Action::EpisodeComplete {
                        scout_index: e.scout_index,
                        reached_goal: e.reached_goal,
                        total_reward: e.total_reward,
                    });
                }
                ServerEvent::TrainingUpdate(e) => {
                    log::info!("TrainingUpdate: ep={}/{}, avg_steps={:.1}",
                        e.episode, e.total_episodes, e.average_steps);
                    state.dispatch(Action::TrainingUpdate {
                        episode: e.episode,
                        total_episodes: e.total_episodes,
                        success_rate: e.success_rate,
                        average_steps: e.average_steps,
                        episode_reward: e.episode_reward,
                    });
                }
                ServerEvent::PolicyUpdate(e) => {
                    log::info!("PolicyUpdate received");
                    state.dispatch(Action::PolicyUpdate { policy: e.policy });
                }
                ServerEvent::TrainingComplete(e) => {
                    log::info!("TrainingComplete: final_success={:.1}%", e.final_success_rate * 100.0);
                    state.dispatch(Action::TrainingComplete {
                        final_success_rate: e.final_success_rate,
                    });
                }
                ServerEvent::Error(e) => {
                    log::error!("Error: {}", e.message);
                    state.dispatch(Action::SetError(Some(e.message)));
                }
            }
        })
    };

    // Handle WebSocket state changes
    let on_ws_state = {
        let ws_state = ws_state.clone();
        Callback::from(move |new_state: WsState| {
            log::info!("WebSocket state: {:?}", new_state);
            ws_state.set(new_state);
        })
    };

    // Auto-connect on mount
    {
        let ws = ws.clone();
        let on_event = on_event.clone();
        let on_ws_state = on_ws_state.clone();
        let ws_state = ws_state.clone();

        use_effect_with((), move |_| {
            if *ws_state == WsState::Disconnected {
                log::info!("Auto-connecting to {}", WS_URL);
                ws.connect(WS_URL, on_event, on_ws_state);
            }
            || ()
        });
    }

    // Connect callback (for manual reconnect)
    let on_connect = {
        let ws = ws.clone();
        let on_event = on_event.clone();
        let on_ws_state = on_ws_state.clone();
        Callback::from(move |_| {
            log::info!("Manual connect to {}", WS_URL);
            ws.connect(WS_URL, on_event.clone(), on_ws_state.clone());
        })
    };

    // Command callback
    let on_command = {
        let ws = ws.clone();
        let state = state.dispatcher();
        Callback::from(move |cmd: ClientCommand| {
            log::info!("Sending command: {:?}", cmd);

            match &cmd {
                ClientCommand::Start { config } => {
                    state.dispatch(Action::StartTraining {
                        grid_size: config.grid_size,
                        n_scouts: config.n_scouts,
                        episodes: config.episodes,
                    });
                }
                ClientCommand::Pause => {
                    state.dispatch(Action::SetPaused(true));
                }
                ClientCommand::Resume => {
                    state.dispatch(Action::SetPaused(false));
                }
                ClientCommand::Stop => {
                    state.dispatch(Action::SetTraining(false));
                }
                _ => {}
            }
            ws.send(cmd);
        })
    };

    // Replay callbacks
    let on_enter_replay = {
        let state = state.dispatcher();
        Callback::from(move |_| state.dispatch(Action::EnterReplayMode))
    };

    let on_exit_replay = {
        let state = state.dispatcher();
        Callback::from(move |_| state.dispatch(Action::ExitReplayMode))
    };

    let on_replay_play = {
        let state = state.dispatcher();
        Callback::from(move |_| state.dispatch(Action::ReplayPlay))
    };

    let on_replay_pause = {
        let state = state.dispatcher();
        Callback::from(move |_| state.dispatch(Action::ReplayPause))
    };

    let on_replay_step = {
        let state = state.dispatcher();
        Callback::from(move |_| state.dispatch(Action::ReplayStep))
    };

    let on_replay_seek = {
        let state = state.dispatcher();
        Callback::from(move |index: usize| state.dispatch(Action::ReplaySeek { index }))
    };

    let on_replay_set_speed = {
        let state = state.dispatcher();
        Callback::from(move |speed: f64| state.dispatch(Action::ReplaySetSpeed { speed }))
    };

    // Replay playback interval using a ref to store the interval handle
    let interval_handle = use_mut_ref(|| None::<gloo::timers::callback::Interval>);
    {
        let state_handle = state.clone();
        let state_dispatcher = state.dispatcher();
        let interval_handle = interval_handle.clone();

        use_effect_with(
            (state_handle.replay_mode, state_handle.replay_playing, state_handle.replay_speed),
            move |(replay_mode, replay_playing, replay_speed)| {
                // Clear any existing interval
                *interval_handle.borrow_mut() = None;

                if !*replay_mode || !*replay_playing {
                    return;
                }

                // Calculate interval based on speed (base interval 50ms at 1x speed)
                let interval_ms = (50.0 / *replay_speed) as u32;

                let state = state_dispatcher.clone();
                let handle = gloo::timers::callback::Interval::new(interval_ms, move || {
                    state.dispatch(Action::ReplayStep);
                });

                // Store handle to keep it alive
                *interval_handle.borrow_mut() = Some(handle);
            }
        );
    }

    html! {
        <div class="app-container">
            <header class="header">
                <h1>{"Many-Eyes"}</h1>
            </header>

            <GridPanel state={(*state).clone()} />

            <div class="bottom-panel">
                <div class="controls-panel">
                    <Controls
                        ws_state={(*ws_state).clone()}
                        training={state.training}
                        paused={state.paused}
                        on_command={on_command}
                        on_connect={on_connect}
                    />
                    <ReplayControls
                        replay_mode={state.replay_mode}
                        replay_playing={state.replay_playing}
                        replay_index={state.replay_index}
                        total_events={state.recorded_events.len()}
                        replay_speed={state.replay_speed}
                        has_recording={!state.recorded_events.is_empty()}
                        on_enter_replay={on_enter_replay}
                        on_exit_replay={on_exit_replay}
                        on_play={on_replay_play}
                        on_pause={on_replay_pause}
                        on_step={on_replay_step}
                        on_seek={on_replay_seek}
                        on_set_speed={on_replay_set_speed}
                    />
                </div>
                <div class="chart-panel">
                    <LearningChart history={state.history.clone()} />
                </div>
                <div class="metrics-panel">
                    <Metrics state={(*state).clone()} />
                </div>
            </div>

            <footer class="footer">
                {"Watch AI scouts learn to navigate from START to GOAL"}
            </footer>
        </div>
    }
}
