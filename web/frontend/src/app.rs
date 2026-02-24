//! Main application component.

use yew::prelude::*;

use crate::components::{Controls, Grid, LearningChart, Metrics, ScoutLegend};
use crate::services::{use_websocket, WsState};
use crate::types::{AppState, ClientCommand, ServerEvent};

const WS_URL: &str = "ws://localhost:3200/ws/train/main";

#[function_component(App)]
pub fn app() -> Html {
    let state = use_state(|| AppState::new(5));
    let ws_state = use_state(|| WsState::Disconnected);
    let ws = use_websocket();

    // Handle server events
    let on_event = {
        let state = state.clone();
        Callback::from(move |event: ServerEvent| {
            let mut new_state = (*state).clone();
            match event {
                ServerEvent::ScoutMove(e) => {
                    // Update scout position
                    if let Some(scout) = new_state.scouts.get_mut(e.scout_index) {
                        scout.position = e.position;
                    }
                    // Mark cell as visited
                    let (row, col) = e.position;
                    if row >= 0 && col >= 0 {
                        if let Some(cell) = new_state
                            .visited_cells
                            .get_mut(row as usize)
                            .and_then(|r| r.get_mut(col as usize))
                        {
                            *cell += 1.0;
                        }
                    }
                }
                ServerEvent::EpisodeComplete(e) => {
                    // Update scout stats
                    if let Some(scout) = new_state.scouts.get_mut(e.scout_index) {
                        scout.total_reward += e.total_reward;
                        scout.episodes_completed += 1;
                        if e.reached_goal {
                            scout.successes += 1;
                        }
                        // Reset position for next episode
                        scout.position = (0, 0);
                    }
                }
                ServerEvent::TrainingUpdate(e) => {
                    new_state.current_episode = e.episode;
                    new_state.total_episodes = e.total_episodes;
                    new_state.success_rate = e.success_rate;
                    new_state.loss = e.loss;
                    new_state.history.success_rates.push(e.success_rate);
                    new_state.history.losses.push(e.loss);
                    new_state.history.episode_rewards.push(e.episode_reward);
                }
                ServerEvent::PolicyUpdate(e) => {
                    new_state.policy = e.policy;
                }
                ServerEvent::TrainingComplete(e) => {
                    new_state.training = false;
                    new_state.success_rate = e.final_success_rate;
                }
                ServerEvent::Error(e) => {
                    new_state.error_message = Some(e.message);
                }
            }
            state.set(new_state);
        })
    };

    // Handle WebSocket state changes
    let on_ws_state = {
        let ws_state = ws_state.clone();
        Callback::from(move |new_state: WsState| {
            ws_state.set(new_state);
        })
    };

    // Connect callback
    let on_connect = {
        let ws = ws.clone();
        let on_event = on_event.clone();
        let on_ws_state = on_ws_state.clone();
        Callback::from(move |_| {
            ws.connect(WS_URL, on_event.clone(), on_ws_state.clone());
        })
    };

    // Command callback
    let on_command = {
        let ws = ws.clone();
        let state = state.clone();
        Callback::from(move |cmd: ClientCommand| {
            match &cmd {
                ClientCommand::Start { config } => {
                    let mut new_state = (*state).clone();
                    new_state.grid_size = config.grid_size;
                    new_state.reset_for_training(config.n_scouts);
                    state.set(new_state);
                }
                ClientCommand::Pause => {
                    let mut new_state = (*state).clone();
                    new_state.paused = true;
                    state.set(new_state);
                }
                ClientCommand::Resume => {
                    let mut new_state = (*state).clone();
                    new_state.paused = false;
                    state.set(new_state);
                }
                ClientCommand::Stop => {
                    let mut new_state = (*state).clone();
                    new_state.training = false;
                    state.set(new_state);
                }
                _ => {}
            }
            ws.send(cmd);
        })
    };

    html! {
        <div class="app-container">
            <header class="header">
                <div>
                    <h1>{"Many-Eyes Learning"}</h1>
                    <div class="subtitle">{"Multi-Scout Reinforcement Learning Visualization"}</div>
                </div>
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
                {"Many-Eyes Learning - Multi-scout exploration for sparse reward environments"}
            </footer>
        </div>
    }
}
