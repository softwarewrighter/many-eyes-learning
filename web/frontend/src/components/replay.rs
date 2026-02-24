//! Replay controls component for playback of recorded training sessions.

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ReplayControlsProps {
    pub replay_mode: bool,
    pub replay_playing: bool,
    pub replay_index: usize,
    pub total_events: usize,
    pub replay_speed: f64,
    pub has_recording: bool,
    pub on_enter_replay: Callback<()>,
    pub on_exit_replay: Callback<()>,
    pub on_play: Callback<()>,
    pub on_pause: Callback<()>,
    pub on_step: Callback<()>,
    pub on_seek: Callback<usize>,
    pub on_set_speed: Callback<f64>,
}

#[function_component(ReplayControls)]
pub fn replay_controls(props: &ReplayControlsProps) -> Html {
    let on_enter = {
        let cb = props.on_enter_replay.clone();
        Callback::from(move |_| cb.emit(()))
    };

    let on_exit = {
        let cb = props.on_exit_replay.clone();
        Callback::from(move |_| cb.emit(()))
    };

    let on_play = {
        let cb = props.on_play.clone();
        Callback::from(move |_| cb.emit(()))
    };

    let on_pause = {
        let cb = props.on_pause.clone();
        Callback::from(move |_| cb.emit(()))
    };

    let on_step = {
        let cb = props.on_step.clone();
        Callback::from(move |_| cb.emit(()))
    };

    let on_restart = {
        let cb = props.on_seek.clone();
        Callback::from(move |_| cb.emit(0))
    };

    let on_seek_change = {
        let cb = props.on_seek.clone();
        let total = props.total_events;
        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                if let Ok(value) = target.value().parse::<usize>() {
                    cb.emit(value.min(total));
                }
            }
        })
    };

    let on_speed_change = {
        let cb = props.on_set_speed.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                if let Ok(value) = target.value().parse::<f64>() {
                    cb.emit(value);
                }
            }
        })
    };

    let progress_percent = if props.total_events > 0 {
        (props.replay_index as f64 / props.total_events as f64 * 100.0) as i32
    } else {
        0
    };

    html! {
        <div class="replay-controls">
            <div class="replay-header">
                <h3>{"Replay"}</h3>
                if props.replay_mode {
                    <button class="btn btn-secondary btn-sm" onclick={on_exit}>
                        {"Exit Replay"}
                    </button>
                } else {
                    <button
                        class="btn btn-primary btn-sm"
                        onclick={on_enter}
                        disabled={!props.has_recording}
                    >
                        {"Enter Replay"}
                    </button>
                }
            </div>

            if props.replay_mode {
                <div class="replay-body">
                    // Playback controls
                    <div class="replay-buttons">
                        <button
                            class="btn btn-icon"
                            onclick={on_restart}
                            title="Restart"
                        >
                            {"\u{23EE}"} // Rewind
                        </button>
                        if props.replay_playing {
                            <button
                                class="btn btn-icon btn-primary"
                                onclick={on_pause}
                                title="Pause"
                            >
                                {"\u{23F8}"} // Pause
                            </button>
                        } else {
                            <button
                                class="btn btn-icon btn-primary"
                                onclick={on_play}
                                title="Play"
                            >
                                {"\u{25B6}"} // Play
                            </button>
                        }
                        <button
                            class="btn btn-icon"
                            onclick={on_step}
                            title="Step Forward"
                            disabled={props.replay_index >= props.total_events}
                        >
                            {"\u{23ED}"} // Step forward
                        </button>
                    </div>

                    // Timeline slider
                    <div class="replay-timeline">
                        <input
                            type="range"
                            min="0"
                            max={props.total_events.to_string()}
                            value={props.replay_index.to_string()}
                            oninput={on_seek_change}
                            class="timeline-slider"
                        />
                        <div class="timeline-label">
                            {format!("{} / {} ({}%)", props.replay_index, props.total_events, progress_percent)}
                        </div>
                    </div>

                    // Speed control
                    <div class="replay-speed">
                        <label>{"Speed: "}{format!("{:.1}x", props.replay_speed)}</label>
                        <input
                            type="range"
                            min="0.1"
                            max="10.0"
                            step="0.1"
                            value={props.replay_speed.to_string()}
                            oninput={on_speed_change}
                            class="speed-slider"
                        />
                    </div>
                </div>
            } else {
                <div class="replay-info">
                    if props.has_recording {
                        {format!("{} events recorded", props.total_events)}
                    } else {
                        {"Run training to record events for replay"}
                    }
                </div>
            }
        </div>
    }
}
