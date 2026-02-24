//! Many-Eyes Learning backend server using Axum.

mod events;
mod trainer;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::Path,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::events::{ClientCommand, ServerEvent};
use crate::trainer::StreamingTrainer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/ws/train/:client_id", get(ws_handler))
        .route("/api/health", get(health))
        .route("/api/grid-info", get(grid_info))
        .nest_service("/", ServeDir::new(frontend_dist_path()))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3200));
    tracing::info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn frontend_dist_path() -> PathBuf {
    // Try relative to binary, then relative to cwd
    let paths = [
        PathBuf::from("../frontend/dist"),
        PathBuf::from("web/frontend/dist"),
        PathBuf::from("frontend/dist"),
    ];

    for path in &paths {
        if path.exists() {
            return path.clone();
        }
    }

    paths[0].clone()
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": "0.1.0"
    }))
}

async fn grid_info() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "default_size": 5,
        "actions": {
            "0": {"name": "up", "arrow": "↑"},
            "1": {"name": "right", "arrow": "→"},
            "2": {"name": "down", "arrow": "↓"},
            "3": {"name": "left", "arrow": "←"}
        },
        "scout_colors": [
            {"id": 0, "name": "Scout 0", "color": "#2196F3"},
            {"id": 1, "name": "Scout 1", "color": "#4CAF50"},
            {"id": 2, "name": "Scout 2", "color": "#FF9800"},
            {"id": 3, "name": "Scout 3", "color": "#9C27B0"},
            {"id": 4, "name": "Scout 4", "color": "#F44336"}
        ]
    }))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(_client_id): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    eprintln!("[WS] Connection opened");
    let (mut sender, mut receiver) = socket.split();

    // Use a channel for control commands during training
    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<ClientCommand>(32);

    // Spawn a task to read messages and forward commands
    let reader_handle = tokio::spawn(async move {
        eprintln!("[WS-Reader] Started");
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    eprintln!("[WS-Reader] Got text: {}", &text[..text.len().min(50)]);
                    if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&text) {
                        eprintln!("[WS-Reader] Parsed command, sending to channel");
                        if cmd_tx.send(cmd).await.is_err() {
                            eprintln!("[WS-Reader] Channel send failed");
                            break;
                        }
                    }
                }
                Ok(Message::Close(reason)) => {
                    eprintln!("[WS-Reader] Close received: {:?}", reason);
                    break;
                }
                Err(e) => {
                    eprintln!("[WS-Reader] Error: {:?}", e);
                    break;
                }
                _ => {}
            }
        }
        eprintln!("[WS-Reader] Finished");
    });

    // Wait for Start command
    eprintln!("[WS] Waiting for commands");
    while let Some(cmd) = cmd_rx.recv().await {
        eprintln!("[WS] Got command from channel");
        match cmd {
            ClientCommand::Start { config } => {
                eprintln!("[WS] Starting training: {:?}", config);
                let mut trainer = StreamingTrainer::new(config);
                let mut event_count = 0;

                loop {
                    // Check for control commands (non-blocking)
                    while let Ok(cmd) = cmd_rx.try_recv() {
                        eprintln!("[WS] Control command: {:?}", cmd);
                        match cmd {
                            ClientCommand::Stop => {
                                eprintln!("[WS] Stop requested");
                                break;
                            }
                            ClientCommand::Pause => trainer.pause(),
                            ClientCommand::Resume => trainer.resume(),
                            ClientCommand::SetSpeed { speed } => trainer.set_speed(speed),
                            _ => {}
                        }
                    }

                    // Get next training event
                    let event = match trainer.step().await {
                        Some(e) => e,
                        None => {
                            eprintln!("[WS] Trainer returned None");
                            break;
                        }
                    };

                    event_count += 1;
                    let json = serde_json::to_string(&event).unwrap();

                    if event_count <= 10 || event_count % 100 == 0 {
                        eprintln!("[WS] Event #{}: {}", event_count, &json[..json.len().min(60)]);
                    }

                    if let Err(e) = sender.send(Message::Text(json)).await {
                        eprintln!("[WS] Send error: {:?}", e);
                        break;
                    }

                    if matches!(event, ServerEvent::TrainingComplete { .. }) {
                        eprintln!("[WS] Training complete after {} events", event_count);
                        break;
                    }
                }
                eprintln!("[WS] Training loop done, {} events sent", event_count);
            }
            _ => {
                eprintln!("[WS] Non-start command: {:?}", cmd);
            }
        }
    }

    eprintln!("[WS] Command channel closed, cleaning up");
    reader_handle.abort();
    eprintln!("[WS] Connection closed");
}
