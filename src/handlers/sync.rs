use axum::{extract::State, http::StatusCode, Json};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::db::AppState;
use crate::models::mesh_node;

#[derive(Serialize)]
pub struct SyncTriggerResponse {
    pub success: bool,
    pub message: String,
    pub applied_count: usize,
}

#[derive(Deserialize)]
pub struct PushTestRequest {
    pub target_instance: String,
    pub entity_type: String,
    pub entity_id: String,
    pub payload: serde_json::Value,
}

/// POST /api/sync/trigger — manually triggers a pull from the relay server
pub async fn trigger_sync(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SyncTriggerResponse>, StatusCode> {
    info!("[API] Manual sync pull triggered");

    match state.sync_engine.pull_and_apply().await {
        Ok(applied) => {
            // Clean up setup account if real users were synced in
            if applied > 0 {
                crate::db::cleanup_setup_if_real_users(&state.db, &state.setup_password).await;
            }
            Ok(Json(SyncTriggerResponse {
                success: true,
                message: "Sync pull completed successfully".to_string(),
                applied_count: applied,
            }))
        }
        Err(e) => Ok(Json(SyncTriggerResponse {
            success: false,
            message: format!("Sync pull failed: {}", e),
            applied_count: 0,
        })),
    }
}

/// POST /api/sync/push_test — manually triggers a push to the relay for testing
pub async fn trigger_push(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PushTestRequest>,
) -> Result<Json<SyncTriggerResponse>, StatusCode> {
    info!(
        "[API] Manual sync push triggered for {}/{}",
        req.entity_type, req.entity_id
    );

    match state
        .sync_engine
        .push_entity(
            &req.target_instance,
            &req.entity_type,
            &req.entity_id,
            &req.payload,
        )
        .await
    {
        Ok(()) => Ok(Json(SyncTriggerResponse {
            success: true,
            message: "Sync push completed successfully".to_string(),
            applied_count: 1,
        })),
        Err(e) => Ok(Json(SyncTriggerResponse {
            success: false,
            message: format!("Sync push failed: {}", e),
            applied_count: 0,
        })),
    }
}

/// POST /api/sync/peers — full sync with all known mesh peers (direct HTTP pull)
pub async fn sync_with_peers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SyncTriggerResponse>, StatusCode> {
    info!("[API] Sync with mesh peers triggered");

    let nodes = mesh_node::Entity::find()
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut total_applied = 0usize;
    let mut errors = Vec::new();

    for node in &nodes {
        if node.base_url.is_empty() {
            continue;
        }
        for entity_type in &["user", "order", "document", "file_resource", "attachment", "item", "order_item_event"] {
            match state
                .sync_engine
                .full_pull_from_peer(&node.base_url, entity_type)
                .await
            {
                Ok(count) => total_applied += count,
                Err(e) => errors.push(format!("{}/{}: {}", node.instance_id, entity_type, e)),
            }
        }
    }

    // Clean up setup account if real users were synced in
    if total_applied > 0 {
        crate::db::cleanup_setup_if_real_users(&state.db, &state.setup_password).await;
    }

    let msg = if errors.is_empty() {
        format!("Synced {} entities from {} peers", total_applied, nodes.len())
    } else {
        format!("Synced {} entities, errors: {}", total_applied, errors.join("; "))
    };

    Ok(Json(SyncTriggerResponse {
        success: errors.is_empty(),
        message: msg,
        applied_count: total_applied,
    }))
}
