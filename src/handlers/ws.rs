use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};

use crate::db::AppState;

/// Shared WebSocket hub state
pub struct WsHub {
    /// Broadcast channel for sending messages to all connected clients
    pub tx: broadcast::Sender<String>,
}

impl WsHub {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { tx }
    }
}

#[derive(Deserialize)]
struct BaseMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(rename = "deviceId", default)]
    device_id: String,
    #[serde(rename = "msgId", default)]
    msg_id: String,
}

#[derive(Serialize)]
struct AckMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(rename = "msgId")]
    msg_id: String,
    status: String,
}

/// GET /E/ws — WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let tx = state.ws_hub.tx.clone();
    let mut rx = tx.subscribe();

    use futures_util::{SinkExt, StreamExt};

    // Generate a temporary web client ID
    let client_id = format!("web_{}", uuid::Uuid::new_v4());
    info!("WebSocket client connected: {}", client_id);

    // Task: forward broadcast messages to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Task: read messages from client and broadcast
    let tx2 = tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    let text_str: &str = &text;
                    // Parse for DEVICE_IDENTIFY handshake
                    if let Ok(base) = serde_json::from_str::<BaseMessage>(text_str) {
                        if base.msg_type == "DEVICE_IDENTIFY" && !base.device_id.is_empty() {
                            info!("Device identified: {}", base.device_id);
                            let ack = AckMessage {
                                msg_type: "ACK".to_string(),
                                msg_id: base.msg_id,
                                status: "connected".to_string(),
                            };
                            if let Ok(ack_json) = serde_json::to_string(&ack) {
                                let _ = tx2.send(ack_json);
                            }
                            continue;
                        }
                    }
                    // Broadcast other messages to all clients
                    let _ = tx2.send(text_str.to_string());
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish, then abort the other
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };

    info!("WebSocket client disconnected: {}", client_id);
}
