use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::db::AppState;

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
        Ok(applied) => Ok(Json(SyncTriggerResponse {
            success: true,
            message: "Sync pull completed successfully".to_string(),
            applied_count: applied,
        })),
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
