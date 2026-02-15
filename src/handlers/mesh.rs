use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;

use crate::db::AppState;

#[derive(Serialize)]
pub struct NodeInfo {
    pub instance_id: String,
    pub role: String,
    pub base_url: String,
    pub weight: i32,
    pub is_online: bool,
}

#[derive(Serialize)]
pub struct MeshStatus {
    pub instance_id: String,
    pub role: String,
    pub base_url: String,
}

/// GET /mesh/nodes — returns list of known peer nodes (currently empty)
pub async fn list_nodes() -> Json<Vec<NodeInfo>> {
    // No peer registry implemented yet — return empty list
    Json(vec![])
}

/// GET /mesh/status — returns this server's identity
pub async fn get_status(State(state): State<Arc<AppState>>) -> Json<MeshStatus> {
    Json(MeshStatus {
        instance_id: state.config.instance_id.clone(),
        role: "peer".to_string(),
        base_url: String::new(),
    })
}
