//! Learning curves chart component using Canvas.

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

use crate::types::TrainingHistory;

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

    html! {
        <div class="panel chart-container">
            <div class="panel-title">{"Learning Curves"}</div>
            <canvas
                ref={canvas_ref}
                class="chart-canvas"
                width="400"
                height="200"
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

    let width = canvas.width() as f64;
    let height = canvas.height() as f64;
    let padding = 40.0;
    let chart_width = width - 2.0 * padding;
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
    ctx.line_to(width - padding, height - padding);
    ctx.stroke();

    // Draw axis labels
    ctx.set_fill_style_str("#a0a0a0");
    ctx.set_font("12px sans-serif");
    ctx.set_text_align("center");
    let _ = ctx.fill_text("Episode", width / 2.0, height - 10.0);

    // Y-axis label at top-left
    ctx.set_text_align("left");
    let _ = ctx.fill_text("Success %", 5.0, padding - 5.0);

    // Draw success rate line
    if !history.success_rates.is_empty() {
        let n = history.success_rates.len();
        let x_scale = chart_width / (n.max(1) - 1).max(1) as f64;

        // Draw line
        ctx.set_stroke_style_str("#e94560");
        ctx.set_line_width(2.0);
        ctx.begin_path();

        for (i, &rate) in history.success_rates.iter().enumerate() {
            let x = padding + i as f64 * x_scale;
            let y = height - padding - rate * chart_height;

            if i == 0 {
                ctx.move_to(x, y);
            } else {
                ctx.line_to(x, y);
            }
        }
        ctx.stroke();

        // Draw points for recent values
        let start_idx = n.saturating_sub(20);
        ctx.set_fill_style_str("#e94560");
        for (i, &rate) in history.success_rates.iter().enumerate().skip(start_idx) {
            let x = padding + i as f64 * x_scale;
            let y = height - padding - rate * chart_height;

            ctx.begin_path();
            ctx.arc(x, y, 3.0, 0.0, std::f64::consts::TAU).ok();
            ctx.fill();
        }

        // Draw current value
        if let Some(&last_rate) = history.success_rates.last() {
            ctx.set_fill_style_str("#eaeaea");
            ctx.set_font("14px sans-serif");
            ctx.set_text_align("right");
            let _ = ctx.fill_text(
                &format!("{:.1}%", last_rate * 100.0),
                width - padding + 5.0,
                height - padding - last_rate * chart_height + 4.0,
            );
        }
    }

    // Draw grid lines and Y-axis labels
    ctx.set_stroke_style_str("#303050");
    ctx.set_line_width(0.5);
    ctx.set_fill_style_str("#707090");
    ctx.set_font("10px sans-serif");
    ctx.set_text_align("right");

    for i in 0..=4 {
        let y = height - padding - (i as f64 / 4.0) * chart_height;

        // Grid line (skip 0)
        if i > 0 {
            ctx.begin_path();
            ctx.move_to(padding, y);
            ctx.line_to(width - padding, y);
            ctx.stroke();
        }

        // Y-axis label
        let _ = ctx.fill_text(&format!("{}%", i * 25), padding - 5.0, y + 3.0);
    }
}
