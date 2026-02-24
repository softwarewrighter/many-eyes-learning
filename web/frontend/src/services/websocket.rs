//! WebSocket service for real-time training communication.

use futures::{channel::mpsc, SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::types::{ClientCommand, ServerEvent};

/// WebSocket connection state
#[derive(Debug, Clone, PartialEq)]
pub enum WsState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Message to send to the WebSocket callback
#[derive(Debug, Clone)]
pub enum WsMessage {
    Connect(String),
    Send(ClientCommand),
    Disconnect,
}

/// WebSocket handle for sending messages
#[derive(Clone)]
pub struct WsHandle {
    sender: Rc<RefCell<Option<mpsc::Sender<String>>>>,
}

impl WsHandle {
    pub fn send(&self, command: ClientCommand) {
        if let Some(ref mut tx) = *self.sender.borrow_mut() {
            if let Ok(json) = serde_json::to_string(&command) {
                let _ = tx.try_send(json);
            }
        }
    }

    pub fn connect(&self, url: &str, on_event: Callback<ServerEvent>, on_state: Callback<WsState>) {
        let sender = self.sender.clone();
        let url = url.to_string();

        on_state.emit(WsState::Connecting);

        spawn_local(async move {
            match WebSocket::open(&url) {
                Ok(ws) => {
                    let (mut write, mut read) = ws.split();
                    let (tx, mut rx) = mpsc::channel::<String>(32);

                    *sender.borrow_mut() = Some(tx);
                    on_state.emit(WsState::Connected);

                    // Spawn writer task
                    spawn_local(async move {
                        while let Some(msg) = rx.next().await {
                            if write.send(Message::Text(msg)).await.is_err() {
                                break;
                            }
                        }
                    });

                    // Read messages
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                if let Ok(event) = serde_json::from_str::<ServerEvent>(&text) {
                                    on_event.emit(event);
                                } else {
                                    log::warn!("Failed to parse event: {}", text);
                                }
                            }
                            Ok(Message::Bytes(_)) => {}
                            Err(e) => {
                                log::error!("WebSocket error: {:?}", e);
                                break;
                            }
                        }
                    }

                    on_state.emit(WsState::Disconnected);
                }
                Err(e) => {
                    on_state.emit(WsState::Error(format!("{:?}", e)));
                }
            }
        });
    }

    pub fn disconnect(&self) {
        *self.sender.borrow_mut() = None;
    }
}

/// Hook for WebSocket connection
#[hook]
pub fn use_websocket() -> WsHandle {
    let sender = use_mut_ref(|| None::<mpsc::Sender<String>>);

    WsHandle {
        sender: sender.clone(),
    }
}
