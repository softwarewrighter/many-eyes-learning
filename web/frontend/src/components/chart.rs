//! Learning curves chart component using Canvas.

use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

use crate::types::TrainingHistory;

const CANVAS_WIDTH: u32 = 500;
const CANVAS_HEIGHT: u32 = 280;

#[derive(Properties, PartialEq)]
pub struct LearningChartProps {
    pub history: TrainingHistory,
}

#[function_component(LearningChart)]
pub fn learning_chart(props: &LearningChartProps) -> Html {
    let canvas_ref = use_node_ref();

    {
        let canvas_ref = canvas_ref.clone();
        let history = props.history.clone();

        use_effect_with(history, move |history| {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                draw_chart(&canvas, history);
            }
            || ()
        });
    }

    // Use CSS to set display size, canvas element gets actual pixel dimensions in draw_chart
    let style = format!("width: {}px; height: {}px;", CANVAS_WIDTH, CANVAS_HEIGHT);

    html! {
        <div class="panel chart-container">
            <div class="panel-title">{"Learning Curves"}</div>
            <canvas
                ref={canvas_ref}
                class="chart-canvas"
                style={style}
            />
        </div>
    }
}

fn draw_chart(canvas: &HtmlCanvasElement, history: &TrainingHistory) {
    let ctx = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok());

    let Some(ctx) = ctx else { return };

    // Get device pixel ratio for sharp rendering on high-DPI displays
    let dpr = window()
        .map(|w| w.device_pixel_ratio())
        .unwrap_or(1.0);

    // Set canvas internal dimensions (scaled for DPI)
    let scaled_width = (CANVAS_WIDTH as f64 * dpr) as u32;
    let scaled_height = (CANVAS_HEIGHT as f64 * dpr) as u32;
    canvas.set_width(scaled_width);
    canvas.set_height(scaled_height);

    // Scale the context to handle DPI
    let _ = ctx.scale(dpr, dpr);

    // Use logical dimensions for drawing
    let width = CANVAS_WIDTH as f64;
    let height = CANVAS_HEIGHT as f64;
    let padding = 50.0;
    let padding_right = 30.0;
    let chart_width = width - padding - padding_right;
    let chart_height = height - 2.0 * padding;

    // Clear canvas
    ctx.set_fill_style_str("#0f3460");
    ctx.fill_rect(0.0, 0.0, width, height);

    // Draw axes
    ctx.set_stroke_style_str("#404060");
    ctx.set_line_width(1.0);
    ctx.begin_path();
    ctx.move_to(padding, padding);
    ctx.line_to(padding, height - padding);
    ctx.line_to(width - padding_right, height - padding);
    ctx.stroke();

    // Draw axis labels
    ctx.set_fill_style_str("#a0a0a0");
    ctx.set_font("12px sans-serif");
    ctx.set_text_align("center");
    let _ = ctx.fill_text("Episode", (padding + width - padding_right) / 2.0, height - 10.0);

    // Y-axis label at top-left
    ctx.set_text_align("left");
    let _ = ctx.fill_text("Avg Steps", 5.0, padding - 5.0);

    // Draw average steps line
    if !history.average_steps.is_empty() {
        let n = history.average_steps.len();
        let x_scale = chart_width / (n.max(1) - 1).max(1) as f64;

        // Calculate max steps for scaling (use max observed, minimum 10)
        let max_steps = history.average_steps
            .iter()
            .cloned()
            .fold(10.0_f64, f64::max)
            .max(10.0);

        // Round up to nice number for y-axis
        let y_max = ((max_steps / 10.0).ceil() * 10.0).max(10.0);

        // Draw grid lines and Y-axis labels first (so line is on top)
        ctx.set_stroke_style_str("#303050");
        ctx.set_line_width(0.5);
        ctx.set_fill_style_str("#707090");
        ctx.set_font("10px sans-serif");
        ctx.set_text_align("right");

        for i in 0..=4 {
            let y = height - padding - (i as f64 / 4.0) * chart_height;
            let label_val = (i as f64 / 4.0) * y_max;

            // Grid line (skip 0)
            if i > 0 {
                ctx.begin_path();
                ctx.move_to(padding, y);
                ctx.line_to(width - padding_right, y);
                ctx.stroke();
            }

            // Y-axis label
            let _ = ctx.fill_text(&format!("{:.0}", label_val), padding - 5.0, y + 3.0);
        }

        // Draw line
        ctx.set_stroke_style_str("#4CAF50");
        ctx.set_line_width(2.0);
        ctx.begin_path();

        for (i, &steps) in history.average_steps.iter().enumerate() {
            let x = padding + i as f64 * x_scale;
            // Lower steps = better, so we want low values at bottom
            let normalized = (steps / y_max).min(1.0);
            let y = height - padding - normalized * chart_height;

            if i == 0 {
                ctx.move_to(x, y);
            } else {
                ctx.line_to(x, y);
            }
        }
        ctx.stroke();

        // Draw points for recent values
        let start_idx = n.saturating_sub(20);
        ctx.set_fill_style_str("#4CAF50");
        for (i, &steps) in history.average_steps.iter().enumerate().skip(start_idx) {
            let x = padding + i as f64 * x_scale;
            let normalized = (steps / y_max).min(1.0);
            let y = height - padding - normalized * chart_height;

            ctx.begin_path();
            ctx.arc(x, y, 3.0, 0.0, std::f64::consts::TAU).ok();
            ctx.fill();
        }

        // Draw current value
        if let Some(&last_steps) = history.average_steps.last() {
            let normalized = (last_steps / y_max).min(1.0);
            let y = height - padding - normalized * chart_height;

            ctx.set_fill_style_str("#eaeaea");
            ctx.set_font("14px sans-serif");
            ctx.set_text_align("left");
            let _ = ctx.fill_text(
                &format!("{:.1}", last_steps),
                width - padding_right + 5.0,
                y + 4.0,
            );
        }
    } else {
        // No data yet - draw empty grid
        ctx.set_stroke_style_str("#303050");
        ctx.set_line_width(0.5);
        ctx.set_fill_style_str("#707090");
        ctx.set_font("10px sans-serif");
        ctx.set_text_align("right");

        for i in 0..=4 {
            let y = height - padding - (i as f64 / 4.0) * chart_height;

            if i > 0 {
                ctx.begin_path();
                ctx.move_to(padding, y);
                ctx.line_to(width - padding_right, y);
                ctx.stroke();
            }

            let _ = ctx.fill_text(&format!("{}", i * 25), padding - 5.0, y + 3.0);
        }
    }
}
