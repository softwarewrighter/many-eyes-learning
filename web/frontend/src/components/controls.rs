//! Training controls component.

use yew::prelude::*;
use web_sys::{HtmlInputElement, HtmlSelectElement};

use crate::types::{ClientCommand, ExplorationMode, TrainingConfig};
use crate::services::WsState;

#[derive(Properties, PartialEq)]
pub struct ControlsProps {
    pub ws_state: WsState,
    pub training: bool,
    pub paused: bool,
    pub on_command: Callback<ClientCommand>,
    pub on_connect: Callback<()>,
}

#[function_component(Controls)]
pub fn controls(props: &ControlsProps) -> Html {
    let n_scouts = use_state(|| 5i32);
    let grid_size = use_state(|| 5i32);
    let episodes = use_state(|| 50i32);
    let speed = use_state(|| 1.0f64);
    let exploration_mode = use_state(|| ExplorationMode::SharedPolicy);

    let on_connect = {
        let on_connect = props.on_connect.clone();
        Callback::from(move |_| on_connect.emit(()))
    };

    let on_start = {
        let on_command = props.on_command.clone();
        let n_scouts = *n_scouts;
        let grid_size = *grid_size;
        let episodes = *episodes;
        let exploration_mode = *exploration_mode;
        Callback::from(move |_| {
            on_command.emit(ClientCommand::Start {
                config: TrainingConfig {
                    n_scouts,
                    grid_size,
                    episodes,
                    steps_per_episode: 100,
                    with_obstacles: false,
                    seed: None,
                    exploration_mode,
                },
            });
        })
    };

    let on_pause = {
        let on_command = props.on_command.clone();
        Callback::from(move |_| on_command.emit(ClientCommand::Pause))
    };

    let on_resume = {
        let on_command = props.on_command.clone();
        Callback::from(move |_| on_command.emit(ClientCommand::Resume))
    };

    let on_stop = {
        let on_command = props.on_command.clone();
        Callback::from(move |_| on_command.emit(ClientCommand::Stop))
    };

    let on_speed_change = {
        let on_command = props.on_command.clone();
        let speed = speed.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                if let Ok(value) = input.value().parse::<f64>() {
                    speed.set(value);
                    on_command.emit(ClientCommand::SetSpeed { speed: value });
                }
            }
        })
    };

    let connected = props.ws_state == WsState::Connected;
    let can_start = connected && !props.training;
    let can_control = connected && props.training;

    html! {
        <div class="panel">
            <div class="panel-title">{"Controls"}</div>
            <div class="controls">
                // Connection status
                <div class="status-indicator">
                    <span class={classes!(
                        "status-dot",
                        connected.then_some("connected"),
                        props.training.then_some("training"),
                    )}></span>
                    <span>{match &props.ws_state {
                        WsState::Disconnected => "Disconnected",
                        WsState::Connecting => "Connecting...",
                        WsState::Connected if props.training => "Training",
                        WsState::Connected => "Ready",
                        WsState::Error(_) => "Error",
                    }}</span>
                </div>

                // Connect button
                if !connected {
                    <button onclick={on_connect} class="primary">
                        {"Connect to Server"}
                    </button>
                }

                // Config (only when not training)
                if !props.training {
                    <div class="control-row full-width">
                        <div class="config-group">
                            <label>{"Scouts (2-10)"}</label>
                            <input
                                type="number"
                                min="2"
                                max="10"
                                value={n_scouts.to_string()}
                                oninput={Callback::from({
                                    let n_scouts = n_scouts.clone();
                                    move |e: InputEvent| {
                                        if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                                            if let Ok(v) = input.value().parse() {
                                                n_scouts.set(v);
                                            }
                                        }
                                    }
                                })}
                                disabled={!can_start}
                            />
                        </div>
                        <div class="config-group">
                            <label>{"Grid Size"}</label>
                            <input
                                type="number"
                                min="3"
                                max="10"
                                value={grid_size.to_string()}
                                oninput={Callback::from({
                                    let grid_size = grid_size.clone();
                                    move |e: InputEvent| {
                                        if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                                            if let Ok(v) = input.value().parse() {
                                                grid_size.set(v);
                                            }
                                        }
                                    }
                                })}
                                disabled={!can_start}
                            />
                        </div>
                        <div class="config-group">
                            <label>{"Episodes"}</label>
                            <input
                                type="number"
                                min="10"
                                max="500"
                                value={episodes.to_string()}
                                oninput={Callback::from({
                                    let episodes = episodes.clone();
                                    move |e: InputEvent| {
                                        if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                                            if let Ok(v) = input.value().parse() {
                                                episodes.set(v);
                                            }
                                        }
                                    }
                                })}
                                disabled={!can_start}
                            />
                        </div>
                    </div>
                    <div class="config-group">
                        <label>{"Exploration Mode"}</label>
                        <select
                            class="exploration-select"
                            onchange={Callback::from({
                                let exploration_mode = exploration_mode.clone();
                                move |e: Event| {
                                    if let Some(select) = e.target_dyn_into::<HtmlSelectElement>() {
                                        let mode = match select.value().as_str() {
                                            "diverse_paths" => ExplorationMode::DiversePaths,
                                            "high_exploration" => ExplorationMode::HighExploration,
                                            "boltzmann" => ExplorationMode::Boltzmann,
                                            _ => ExplorationMode::SharedPolicy,
                                        };
                                        exploration_mode.set(mode);
                                    }
                                }
                            })}
                            disabled={!can_start}
                        >
                            <option value="shared_policy" selected={*exploration_mode == ExplorationMode::SharedPolicy}>
                                {"Shared Policy"}
                            </option>
                            <option value="diverse_paths" selected={*exploration_mode == ExplorationMode::DiversePaths}>
                                {"Diverse Paths"}
                            </option>
                            <option value="high_exploration" selected={*exploration_mode == ExplorationMode::HighExploration}>
                                {"High Exploration"}
                            </option>
                            <option value="boltzmann" selected={*exploration_mode == ExplorationMode::Boltzmann}>
                                {"Boltzmann (Softmax)"}
                            </option>
                        </select>
                    </div>
                }

                // Training controls
                <div class="control-row">
                    if !props.training {
                        <button onclick={on_start} disabled={!can_start} class="primary">
                            {"Start Training"}
                        </button>
                    } else {
                        if props.paused {
                            <button onclick={on_resume} disabled={!can_control}>
                                {"Resume"}
                            </button>
                        } else {
                            <button onclick={on_pause} disabled={!can_control}>
                                {"Pause"}
                            </button>
                        }
                        <button onclick={on_stop} disabled={!can_control}>
                            {"Stop"}
                        </button>
                    }
                </div>

                // Speed control
                if props.training {
                    <div class="config-group">
                        <label>{format!("Speed: {}x", *speed)}</label>
                        <input
                            type="range"
                            min="1"
                            max="100"
                            step="1"
                            value={speed.to_string()}
                            oninput={on_speed_change}
                        />
                    </div>
                }
            </div>
        </div>
    }
}
