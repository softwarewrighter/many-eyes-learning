//! Many-Eyes Learning Web Visualization
//!
//! A Yew/WASM frontend for real-time visualization of multi-scout
//! reinforcement learning training.

mod app;
mod components;
mod services;
mod types;

use wasm_bindgen::prelude::*;

/// Entry point for the WASM application.
#[wasm_bindgen(start)]
pub fn run() {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize logging
    wasm_logger::init(wasm_logger::Config::default());

    // Mount the app
    yew::Renderer::<app::App>::new().render();
}
