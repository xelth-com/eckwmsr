use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use sea_orm::EntityTrait;
use tokio::time::{interval, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{info, warn};

use crate::db::AppState;
use crate::handlers::mesh_ws::MeshSignal;
use crate::models::mesh_node;

/// Background loop that periodically connects to known mesh peers via WebSocket.
/// This enables nodes behind NAT to maintain bidirectional communication.
pub async fn start_outbound_ws_loop(state: Arc<AppState>) {
    let mut ticker = interval(Duration::from_secs(30));

    loop {
        ticker.tick().await;

        let nodes = match mesh_node::Entity::find().all(&state.db).await {
            Ok(n) => n,
            Err(e) => {
                tracing::error!("WS Client: failed to fetch mesh nodes: {}", e);
                continue;
            }
        };

        let my_id = &state.config.instance_id;

        for node in nodes {
            if node.base_url.is_empty() || node.instance_id == *my_id {
                continue;
            }

            if state.mesh_hub.is_peer_connected(&node.instance_id) {
                continue;
            }

            let ws_url = match base_url_to_ws(&node.base_url, my_id) {
                Some(u) => u,
                None => continue,
            };

            let state_clone = state.clone();
            let peer_id = node.instance_id.clone();
            let my_id_owned = my_id.clone();

            tokio::spawn(async move {
                handle_outbound_connection(state_clone, &ws_url, &peer_id, &my_id_owned).await;
            });
        }
    }
}

/// Convert an HTTP(S) base_url to a WebSocket URL targeting /mesh/ws
fn base_url_to_ws(base_url: &str, instance_id: &str) -> Option<String> {
    let ws = if base_url.starts_with("https://") {
        base_url.replacen("https://", "wss://", 1)
    } else if base_url.starts_with("http://") {
        base_url.replacen("http://", "ws://", 1)
    } else {
        return None;
    };
    let ws = ws.trim_end_matches('/');
    Some(format!("{}/mesh/ws?instance_id={}", ws, instance_id))
}

async fn handle_outbound_connection(
    state: Arc<AppState>,
    ws_url: &str,
    peer_id: &str,
    my_id: &str,
) {
    let (ws_stream, _) = match connect_async(ws_url).await {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!("Outbound WS to {} failed: {}", peer_id, e);
            return;
        }
    };

    info!("Outbound WS connected to peer: {}", peer_id);
    state.mesh_hub.register(peer_id);

    let (mut write, mut read) = ws_stream.split();
    let mut hub_rx = state.mesh_hub.subscribe();

    // Send HELLO
    let hello = MeshSignal {
        signal_type: "HELLO".into(),
        sender_id: my_id.to_string(),
        entity_type: None,
        entity_id: None,
    };
    let _ = write
        .send(Message::Text(serde_json::to_string(&hello).unwrap()))
        .await;

    // Forward hub broadcasts to peer
    let peer_id_send = peer_id.to_string();
    let mut send_task = tokio::spawn(async move {
        while let Ok(signal) = hub_rx.recv().await {
            if signal.sender_id == peer_id_send {
                continue;
            }
            if let Ok(json) = serde_json::to_string(&signal) {
                if write.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Read incoming messages from peer
    let state_read = state.clone();
    let peer_id_recv = peer_id.to_string();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if let Message::Text(text) = msg {
                if let Ok(signal) = serde_json::from_str::<MeshSignal>(&text) {
                    match signal.signal_type.as_str() {
                        "PING" => {
                            info!("Outbound WS: PING from {}", peer_id_recv);
                        }
                        "UPDATE" => {
                            let et = signal.entity_type.clone().unwrap_or_default();
                            info!("Outbound WS: UPDATE from {} — {}", peer_id_recv, et);
                            state_read.mesh_hub.broadcast(signal);

                            if !et.is_empty() {
                                let pid = peer_id_recv.clone();
                                let sync_state = state_read.clone();
                                let entity = et.clone();
                                tokio::spawn(async move {
                                    if let Ok(Some(node)) =
                                        mesh_node::Entity::find_by_id(&pid)
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
                                                    pid,
                                                    e
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
        }
    });

    tokio::select! {
        _ = &mut send_task => { recv_task.abort(); }
        _ = &mut recv_task => { send_task.abort(); }
    }

    state.mesh_hub.unregister(peer_id);
    warn!("Outbound WS disconnected from peer: {}", peer_id);
}
