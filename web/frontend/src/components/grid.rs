//! SVG Grid visualization component.

use yew::prelude::*;

use crate::types::{AppState, ScoutInfo};

/// Scout colors (colorblind-friendly)
const SCOUT_COLORS: [&str; 5] = [
    "#2196F3", // Blue
    "#4CAF50", // Green
    "#FF9800", // Orange
    "#9C27B0", // Purple
    "#F44336", // Red
];

/// Action arrows
const ACTION_ARROWS: [&str; 4] = [
    "\u{2191}", // Up
    "\u{2192}", // Right
    "\u{2193}", // Down
    "\u{2190}", // Left
];

#[derive(Properties, PartialEq)]
pub struct GridProps {
    pub state: AppState,
}

#[function_component(Grid)]
pub fn grid(props: &GridProps) -> Html {
    let state = &props.state;
    let grid_size = state.grid_size as usize;
    let cell_size = 60;
    let svg_size = grid_size * cell_size;

    // Normalize visit counts for opacity
    let max_visits: f64 = state
        .visited_cells
        .iter()
        .flat_map(|row| row.iter())
        .cloned()
        .fold(1.0, f64::max);

    html! {
        <div class="grid-container">
            <div class="grid-title">
                {"Grid World "}
                <span style="color: var(--text-secondary);">
                    {format!("({}x{})", grid_size, grid_size)}
                </span>
            </div>
            <svg
                class="grid-svg"
                width={svg_size.to_string()}
                height={svg_size.to_string()}
                viewBox={format!("0 0 {} {}", svg_size, svg_size)}
            >
                // Cells
                {for (0..grid_size).flat_map(|row| {
                    (0..grid_size).map(move |col| {
                        let x = col * cell_size;
                        let y = row * cell_size;
                        let is_start = row == 0 && col == 0;
                        let is_goal = row == grid_size - 1 && col == grid_size - 1;

                        // Visit intensity
                        let visits = state.visited_cells.get(row)
                            .and_then(|r| r.get(col))
                            .copied()
                            .unwrap_or(0.0);
                        let visit_opacity = if visits > 0.0 {
                            0.2 + 0.6 * (visits / max_visits)
                        } else {
                            0.0
                        };

                        // Find if a scout is here
                        let scout_here: Option<&ScoutInfo> = state.scouts.iter()
                            .find(|s| s.position == (row as i32, col as i32));

                        // Get policy action for this cell
                        let policy_action = state.policy.get(row)
                            .and_then(|r| r.get(col))
                            .copied();

                        html! {
                            <g key={format!("{}-{}", row, col)}>
                                // Cell background
                                <rect
                                    x={x.to_string()}
                                    y={y.to_string()}
                                    width={cell_size.to_string()}
                                    height={cell_size.to_string()}
                                    class={classes!(
                                        "grid-cell",
                                        is_start.then_some("start"),
                                        is_goal.then_some("goal"),
                                    )}
                                />
                                // Visit overlay
                                if visits > 0.0 && !is_goal {
                                    <rect
                                        x={x.to_string()}
                                        y={y.to_string()}
                                        width={cell_size.to_string()}
                                        height={cell_size.to_string()}
                                        fill="#e94560"
                                        fill-opacity={visit_opacity.to_string()}
                                    />
                                }
                                // Policy arrow (only show after policy is learned)
                                if state.policy_received {
                                    if let Some(action) = policy_action {
                                        if !is_goal && !is_start {
                                            <text
                                                x={(x + cell_size / 2).to_string()}
                                                y={(y + cell_size / 2 + 4).to_string()}
                                                class="policy-arrow"
                                                text-anchor="middle"
                                                font-size="18"
                                            >
                                                {ACTION_ARROWS.get(action as usize).unwrap_or(&"?")}
                                            </text>
                                        }
                                    }
                                }
                                // Start label
                                if is_start {
                                    <text
                                        x={(x + cell_size / 2).to_string()}
                                        y={(y + cell_size - 8).to_string()}
                                        text-anchor="middle"
                                        fill="#2196F3"
                                        font-size="10"
                                        font-weight="bold"
                                    >
                                        {"START"}
                                    </text>
                                }
                                // Goal star
                                if is_goal {
                                    <text
                                        x={(x + cell_size / 2).to_string()}
                                        y={(y + cell_size / 2 + 6).to_string()}
                                        text-anchor="middle"
                                        fill="white"
                                        font-size="24"
                                    >
                                        {"\u{2605}"}
                                    </text>
                                    <text
                                        x={(x + cell_size / 2).to_string()}
                                        y={(y + cell_size - 8).to_string()}
                                        text-anchor="middle"
                                        fill="white"
                                        font-size="10"
                                        font-weight="bold"
                                    >
                                        {"GOAL"}
                                    </text>
                                }
                                // Scout marker
                                if let Some(scout) = scout_here {
                                    <circle
                                        cx={(x + cell_size / 2).to_string()}
                                        cy={(y + cell_size / 2 - 5).to_string()}
                                        r="12"
                                        fill={*SCOUT_COLORS.get(scout.index % 5).unwrap_or(&"#888")}
                                        class="scout-marker"
                                        stroke="white"
                                        stroke-width="2"
                                    />
                                    <text
                                        x={(x + cell_size / 2).to_string()}
                                        y={(y + cell_size / 2 - 1).to_string()}
                                        text-anchor="middle"
                                        fill="white"
                                        font-size="10"
                                        font-weight="bold"
                                    >
                                        {scout.index.to_string()}
                                    </text>
                                }
                            </g>
                        }
                    })
                })}
            </svg>
        </div>
    }
}
