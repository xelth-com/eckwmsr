use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tracing::info;

use crate::db::AppState;
use crate::models::mesh_node;

/// Signals exchanged between mesh nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshSignal {
    #[serde(rename = "type")]
    pub signal_type: String, // "UPDATE", "PING", "HELLO"
    #[serde(rename = "senderId")]
    pub sender_id: String,
    #[serde(rename = "entityType", skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    #[serde(rename = "entityId", skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<String>,
}

/// Manages active WebSocket connections to other mesh nodes
#[derive(Clone)]
pub struct MeshHub {
    tx: broadcast::Sender<MeshSignal>,
    connected_peers: Arc<Mutex<HashMap<String, bool>>>,
}

impl MeshHub {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            tx,
            connected_peers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Broadcast a signal to all connected peers
    pub fn broadcast(&self, signal: MeshSignal) {
        let _ = self.tx.send(signal);
    }

    /// Notify peers about an entity update
    pub fn notify_update(&self, sender_id: &str, entity_type: &str, entity_id: &str) {
        self.broadcast(MeshSignal {
            signal_type: "UPDATE".into(),
            sender_id: sender_id.to_string(),
            entity_type: Some(entity_type.to_string()),
            entity_id: Some(entity_id.to_string()),
        });
    }

    pub fn register(&self, instance_id: &str) {
        if let Ok(mut peers) = self.connected_peers.lock() {
            peers.insert(instance_id.to_string(), true);
        }
    }

    pub fn unregister(&self, instance_id: &str) {
        if let Ok(mut peers) = self.connected_peers.lock() {
            peers.remove(instance_id);
        }
    }

    pub fn peer_count(&self) -> usize {
        self.connected_peers.lock().map(|p| p.len()).unwrap_or(0)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<MeshSignal> {
        self.tx.subscribe()
    }
}

#[derive(Deserialize)]
pub struct MeshWsParams {
    pub instance_id: String,
}

/// GET /E/mesh/ws?instance_id=...
pub async fn mesh_ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<MeshWsParams>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_mesh_socket(socket, params.instance_id, state))
}

async fn handle_mesh_socket(socket: WebSocket, peer_id: String, state: Arc<AppState>) {
    info!("Mesh WS: peer connected: {}", peer_id);
    state.mesh_hub.register(&peer_id);

    let (mut ws_tx, mut ws_rx) = socket.split();
    let mut hub_rx = state.mesh_hub.subscribe();
    let my_id = state.config.instance_id.clone();

    // Send HELLO
    let hello = MeshSignal {
        signal_type: "HELLO".into(),
        sender_id: my_id.clone(),
        entity_type: None,
        entity_id: None,
    };
    let _ = ws_tx
        .send(Message::Text(serde_json::to_string(&hello).unwrap()))
        .await;

    // Forward hub broadcasts to this peer's WebSocket
    let peer_id_send = peer_id.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(signal) = hub_rx.recv().await {
            // Don't echo signals back to sender
            if signal.sender_id == peer_id_send {
                continue;
            }
            if let Ok(json) = serde_json::to_string(&signal) {
                if ws_tx.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Read incoming messages from peer
    let state_read = state.clone();
    let peer_id_recv = peer_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(signal) = serde_json::from_str::<MeshSignal>(&text) {
                        match signal.signal_type.as_str() {
                            "PING" => {
                                // Respond with PONG via hub (or direct, but hub is simpler)
                                info!("Mesh WS: PING from {}", peer_id_recv);
                            }
                            "UPDATE" => {
                                let et = signal.entity_type.clone().unwrap_or_default();
                                info!(
                                    "Mesh WS: UPDATE from {} — {}",
                                    peer_id_recv, et
                                );
                                // Re-broadcast to other peers
                                state_read.mesh_hub.broadcast(signal);

                                // Trigger sync with this peer
                                if !et.is_empty() {
                                    let pid = peer_id_recv.clone();
                                    let sync_state = state_read.clone();
                                    let entity = et.clone();
                                    tokio::spawn(async move {
                                        // Look up peer's base_url from mesh_nodes
                                        if let Ok(Some(node)) = mesh_node::Entity::find_by_id(&pid)
                                            .one(&sync_state.db)
                                            .await
                                        {
                                            if !node.base_url.is_empty() {
                                                if let Err(e) = sync_state
                                                    .sync_engine
                                                    .sync_with_peer(&node.base_url, &entity)
                                                    .await
                                                {
                                                    tracing::error!(
                                                        "Mesh sync with {} failed: {}",
                                                        pid, e
                                                    );
                                                }
                                            }
                                        }
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish (connection closed from either side)
    tokio::select! {
        _ = &mut send_task => { recv_task.abort(); }
        _ = &mut recv_task => { send_task.abort(); }
    }

    state.mesh_hub.unregister(&peer_id);
    info!("Mesh WS: peer disconnected: {}", peer_id);
}
