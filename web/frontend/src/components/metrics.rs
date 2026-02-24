//! Metrics panel component.

use yew::prelude::*;

use crate::types::AppState;

#[derive(Properties, PartialEq)]
pub struct MetricsProps {
    pub state: AppState,
}

#[function_component(Metrics)]
pub fn metrics(props: &MetricsProps) -> Html {
    let state = &props.state;

    let progress = if state.total_episodes > 0 {
        (state.current_episode as f64 / state.total_episodes as f64 * 100.0) as i32
    } else {
        0
    };

    html! {
        <div class="panel">
            <div class="panel-title">{"Training Metrics"}</div>
            <div class="metrics-grid">
                <div class="metric">
                    <div class="metric-value">
                        {format!("{}/{}", state.current_episode, state.total_episodes)}
                    </div>
                    <div class="metric-label">{"Episode"}</div>
                </div>
                <div class={classes!("metric", (state.success_rate > 0.5).then_some("success"))}>
                    <div class="metric-value">
                        {format!("{:.1}%", state.success_rate * 100.0)}
                    </div>
                    <div class="metric-label">{"Success Rate"}</div>
                </div>
                <div class="metric">
                    <div class="metric-value">
                        {format!("{:.1}", state.average_steps)}
                    </div>
                    <div class="metric-label">{"Avg Steps"}</div>
                </div>
                <div class="metric">
                    <div class="metric-value">
                        {format!("{}%", progress)}
                    </div>
                    <div class="metric-label">{"Progress"}</div>
                </div>
            </div>

            // Error message
            if let Some(ref error) = state.error_message {
                <div style="color: var(--error); margin-top: 0.5rem; font-size: 0.85rem;">
                    {error}
                </div>
            }
        </div>
    }
}
