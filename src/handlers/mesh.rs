use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{EntityTrait, QueryOrder};
use serde::Serialize;
use std::sync::Arc;

use crate::db::AppState;
use crate::models::mesh_node;
use crate::sync::relay_client::RelayClient;

#[derive(Serialize)]
pub struct NodeInfo {
    pub instance_id: String,
    pub name: String,
    pub role: String,
    pub base_url: String,
    pub is_online: bool,
}

#[derive(Serialize)]
pub struct MeshStatus {
    pub instance_id: String,
    pub mesh_id: String,
    pub role: String,
}

/// GET /mesh/nodes — returns list of locally paired mesh nodes
pub async fn list_nodes(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<NodeInfo>> {
    let nodes = mesh_node::Entity::find()
        .order_by_asc(mesh_node::Column::Name)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let info_list = nodes
        .into_iter()
        .map(|n| NodeInfo {
            instance_id: n.instance_id,
            name: n.name,
            role: n.role,
            base_url: n.base_url,
            is_online: n.status == "active",
        })
        .collect();

    Json(info_list)
}

/// DELETE /api/admin/mesh/:id — removes a mesh node
pub async fn delete_node(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = mesh_node::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected == 0 {
        return Err((StatusCode::NOT_FOUND, "Node not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

/// GET /mesh/status — returns this server's identity
pub async fn get_status(State(state): State<Arc<AppState>>) -> Json<MeshStatus> {
    Json(MeshStatus {
        instance_id: state.config.instance_id.clone(),
        mesh_id: state.config.mesh_id.clone(),
        role: "peer".to_string(),
    })
}

/// GET /mesh/relay-status — queries the relay for live mesh status
pub async fn get_relay_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<crate::sync::relay_client::RelayNodeInfo>>, (StatusCode, String)> {
    let relay = RelayClient::new(
        &state.config.sync_relay_url,
        &state.config.instance_id,
        &state.config.mesh_id,
    );

    let nodes = relay
        .get_mesh_status()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    Ok(Json(nodes))
}
