//! Grid panel containing all scout grids.

use yew::prelude::*;

use crate::components::SingleScoutGrid;
use crate::types::AppState;

#[derive(Properties, PartialEq)]
pub struct GridPanelProps {
    pub state: AppState,
}

#[function_component(GridPanel)]
pub fn grid_panel(props: &GridPanelProps) -> Html {
    let state = &props.state;
    let n_scouts = state.n_scouts as usize;

    // Add class for two-row layout when more than 5 scouts
    let row_class = if n_scouts > 5 { "grids-row two-rows" } else { "grids-row" };

    html! {
        <div class="grid-panel">
            <div class={row_class}>
                {for (0..n_scouts).map(|i| {
                    // Get scout position, defaulting to (0, 0) if scout doesn't exist
                    let scout_position = state.scouts.get(i)
                        .map(|s| s.position)
                        .unwrap_or((0, 0));

                    // Get per-scout visited cells, defaulting to empty grid
                    let visited_cells = state.per_scout_visited_cells.get(i)
                        .cloned()
                        .unwrap_or_else(|| {
                            let size = state.grid_size as usize;
                            vec![vec![0.0; size]; size]
                        });

                    html! {
                        <SingleScoutGrid
                            key={i}
                            scout_index={i}
                            scout_position={scout_position}
                            grid_size={state.grid_size}
                            visited_cells={visited_cells}
                            policy={state.policy.clone()}
                            policy_received={state.policy_received}
                            n_scouts={state.n_scouts}
                        />
                    }
                })}
            </div>
        </div>
    }
}
