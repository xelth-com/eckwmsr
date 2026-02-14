use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use std::sync::Arc;
use tracing::info;

use crate::db::AppState;

#[derive(Serialize)]
pub struct SyncTriggerResponse {
    pub success: bool,
    pub message: String,
    pub applied_count: usize,
}

/// POST /api/sync/trigger — manually triggers a pull from the relay server
pub async fn trigger_sync(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SyncTriggerResponse>, StatusCode> {
    info!("[API] Manual sync triggered");

    match state.sync_engine.pull_and_apply().await {
        Ok(applied) => Ok(Json(SyncTriggerResponse {
            success: true,
            message: "Sync completed successfully".to_string(),
            applied_count: applied,
        })),
        Err(e) => Ok(Json(SyncTriggerResponse {
            success: false,
            message: format!("Sync failed: {}", e),
            applied_count: 0,
        })),
    }
}
