//! Single scout grid visualization component.

use yew::prelude::*;

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
pub struct SingleScoutGridProps {
    pub scout_index: usize,
    pub scout_position: (i32, i32),
    pub grid_size: i32,
    pub visited_cells: Vec<Vec<f64>>,  // Per-scout visited cells
    pub policy: Vec<Vec<i32>>,
    pub policy_received: bool,
}

#[function_component(SingleScoutGrid)]
pub fn single_scout_grid(props: &SingleScoutGridProps) -> Html {
    let grid_size = props.grid_size as usize;
    let cell_size = 40;  // Smaller cells to fit 5 grids
    let svg_size = grid_size * cell_size;
    let scout_color = SCOUT_COLORS.get(props.scout_index % 5).unwrap_or(&"#888");

    // Normalize visit counts for opacity
    let max_visits: f64 = props
        .visited_cells
        .iter()
        .flat_map(|row| row.iter())
        .cloned()
        .fold(1.0, f64::max);

    // Scout 0 is always random (exploration baseline)
    let label = if props.scout_index == 0 {
        "Random".to_string()
    } else {
        format!("Scout {}", props.scout_index)
    };

    html! {
        <div class="single-scout-grid">
            <div class="scout-grid-header">
                <span class="scout-indicator" style={format!("background-color: {}", scout_color)}></span>
                <span class="scout-label">{label}</span>
            </div>
            <svg
                class="grid-svg-small"
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

                        // Visit intensity using scout's color
                        let visits = props.visited_cells.get(row)
                            .and_then(|r| r.get(col))
                            .copied()
                            .unwrap_or(0.0);
                        let visit_opacity = if visits > 0.0 {
                            0.2 + 0.6 * (visits / max_visits)
                        } else {
                            0.0
                        };

                        // Check if this scout is at this position
                        let scout_here = props.scout_position == (row as i32, col as i32);

                        // Get policy action for this cell
                        let policy_action = props.policy.get(row)
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
                                // Visit overlay with scout's color
                                if visits > 0.0 && !is_goal {
                                    <rect
                                        x={x.to_string()}
                                        y={y.to_string()}
                                        width={cell_size.to_string()}
                                        height={cell_size.to_string()}
                                        fill={*scout_color}
                                        fill-opacity={visit_opacity.to_string()}
                                    />
                                }
                                // Policy arrow (only show after policy is learned)
                                if props.policy_received {
                                    if let Some(action) = policy_action {
                                        if !is_goal && !is_start {
                                            <text
                                                x={(x + cell_size / 2).to_string()}
                                                y={(y + cell_size / 2 + 3).to_string()}
                                                class="policy-arrow"
                                                text-anchor="middle"
                                                font-size="12"
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
                                        y={(y + cell_size - 4).to_string()}
                                        text-anchor="middle"
                                        fill="#2196F3"
                                        font-size="7"
                                        font-weight="bold"
                                    >
                                        {"S"}
                                    </text>
                                }
                                // Goal star
                                if is_goal {
                                    <text
                                        x={(x + cell_size / 2).to_string()}
                                        y={(y + cell_size / 2 + 4).to_string()}
                                        text-anchor="middle"
                                        fill="white"
                                        font-size="16"
                                    >
                                        {"\u{2605}"}
                                    </text>
                                }
                                // Scout marker
                                if scout_here {
                                    <circle
                                        cx={(x + cell_size / 2).to_string()}
                                        cy={(y + cell_size / 2 - 3).to_string()}
                                        r="8"
                                        fill={*scout_color}
                                        class="scout-marker"
                                        stroke="white"
                                        stroke-width="2"
                                    />
                                    <text
                                        x={(x + cell_size / 2).to_string()}
                                        y={(y + cell_size / 2 + 1).to_string()}
                                        text-anchor="middle"
                                        fill="white"
                                        font-size="8"
                                        font-weight="bold"
                                    >
                                        {props.scout_index.to_string()}
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
