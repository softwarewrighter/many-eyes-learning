//! Main application component.

use std::rc::Rc;
use yew::prelude::*;

use crate::components::{Controls, Grid, LearningChart, Metrics, ScoutLegend};
use crate::services::{use_websocket, WsState};
use crate::types::{AppState, ClientCommand, ServerEvent, ScoutInfo, TrainingHistory};

const WS_URL: &str = "ws://localhost:3200/ws/train/main";

/// Actions for state reducer
pub enum Action {
    ScoutMove { scout_index: usize, position: (i32, i32) },
    EpisodeComplete { scout_index: usize, reached_goal: bool, total_reward: f64 },
    TrainingUpdate { episode: i32, total_episodes: i32, success_rate: f64, loss: f64, episode_reward: f64 },
    PolicyUpdate { policy: Vec<Vec<i32>> },
    TrainingComplete { final_success_rate: f64 },
    StartTraining { grid_size: i32, n_scouts: i32, episodes: i32 },
    SetPaused(bool),
    SetTraining(bool),
    SetError(Option<String>),
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

                // Mark cell as visited
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
            }
            Action::TrainingUpdate { episode, total_episodes, success_rate, loss, episode_reward } => {
                new_state.current_episode = episode;
                new_state.total_episodes = total_episodes;
                new_state.success_rate = success_rate;
                new_state.loss = loss;
                new_state.history.success_rates.push(success_rate);
                new_state.history.losses.push(loss);
                new_state.history.episode_rewards.push(episode_reward);
            }
            Action::PolicyUpdate { policy } => {
                new_state.policy = policy;
                new_state.policy_received = true;
            }
            Action::TrainingComplete { final_success_rate } => {
                new_state.training = false;
                new_state.success_rate = final_success_rate;
            }
            Action::StartTraining { grid_size, n_scouts, episodes } => {
                new_state.grid_size = grid_size;
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
                new_state.visited_cells = vec![vec![0.0; size]; size];
                new_state.policy = vec![vec![0; size]; size];
                new_state.policy_received = false;
                new_state.current_episode = 0;
                new_state.success_rate = 0.0;
                new_state.loss = 0.0;
                new_state.history = TrainingHistory::default();
                new_state.error_message = None;
                new_state.training = true;
                new_state.paused = false;
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
                ServerEvent::EpisodeComplete(e) => {
                    log::info!("EpisodeComplete: scout={}, goal={}", e.scout_index, e.reached_goal);
                    state.dispatch(Action::EpisodeComplete {
                        scout_index: e.scout_index,
                        reached_goal: e.reached_goal,
                        total_reward: e.total_reward,
                    });
                }
                ServerEvent::TrainingUpdate(e) => {
                    log::info!("TrainingUpdate: ep={}/{}, success={:.1}%",
                        e.episode, e.total_episodes, e.success_rate * 100.0);
                    state.dispatch(Action::TrainingUpdate {
                        episode: e.episode,
                        total_episodes: e.total_episodes,
                        success_rate: e.success_rate,
                        loss: e.loss,
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

    html! {
        <div class="app-container">
            <header class="header">
                <h1>{"Many-Eyes"}</h1>
            </header>

            <main class="main-area">
                <Grid state={(*state).clone()} />
                <LearningChart history={state.history.clone()} />
            </main>

            <aside class="side-panel">
                <Controls
                    ws_state={(*ws_state).clone()}
                    training={state.training}
                    paused={state.paused}
                    on_command={on_command}
                    on_connect={on_connect}
                />
                <Metrics state={(*state).clone()} />
                <ScoutLegend scouts={state.scouts.clone()} />
            </aside>

            <footer class="footer">
                {"Watch AI scouts learn to navigate from START to GOAL"}
            </footer>
        </div>
    }
}
