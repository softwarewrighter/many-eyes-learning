//! Scout legend component.

use yew::prelude::*;

use crate::types::ScoutInfo;

/// Scout colors (colorblind-friendly)
const SCOUT_COLORS: [&str; 5] = [
    "#2196F3", // Blue
    "#4CAF50", // Green
    "#FF9800", // Orange
    "#9C27B0", // Purple
    "#F44336", // Red
];

fn get_scout_type(index: usize, total: usize) -> &'static str {
    if index == 0 {
        "Random"
    } else {
        let epsilon = 0.1 + 0.2 * (index as f64 / total as f64);
        if epsilon < 0.2 {
            "Epsilon-Greedy (low)"
        } else if epsilon < 0.3 {
            "Epsilon-Greedy (med)"
        } else {
            "Epsilon-Greedy (high)"
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct ScoutLegendProps {
    pub scouts: Vec<ScoutInfo>,
}

#[function_component(ScoutLegend)]
pub fn scout_legend(props: &ScoutLegendProps) -> Html {
    let total = props.scouts.len();

    html! {
        <div class="panel">
            <div class="panel-title">{"Scouts"}</div>
            <div class="scout-legend">
                {for props.scouts.iter().map(|scout| {
                    let color = SCOUT_COLORS.get(scout.index % 5).unwrap_or(&"#888");
                    let success_rate = if scout.episodes_completed > 0 {
                        scout.successes as f64 / scout.episodes_completed as f64 * 100.0
                    } else {
                        0.0
                    };

                    html! {
                        <div class="scout-item" key={scout.index}>
                            <div
                                class="scout-color"
                                style={format!("background-color: {}", color)}
                            />
                            <div class="scout-info">
                                <div class="scout-name">{format!("Scout {}", scout.index)}</div>
                                <div class="scout-type">{get_scout_type(scout.index, total)}</div>
                            </div>
                            <div class="scout-stats">
                                <div class="scout-reward">
                                    {format!("{:.1}", scout.total_reward)}
                                </div>
                                <div class="scout-status">
                                    {format!("{:.0}% success", success_rate)}
                                </div>
                            </div>
                        </div>
                    }
                })}
                if props.scouts.is_empty() {
                    <div style="color: var(--text-secondary); text-align: center; padding: 1rem;">
                        {"No scouts active"}
                    </div>
                }
            </div>
        </div>
    }
}
