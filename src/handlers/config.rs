use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use std::sync::Arc;
use tokio::fs;
use tracing::{error, info};

use crate::db::AppState;

#[derive(Deserialize)]
pub struct SaveKeyRequest {
    pub network_key: String,
}

/// POST /api/admin/config/save-key
/// Updates the SYNC_NETWORK_KEY in the .env file.
pub async fn save_network_key(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<SaveKeyRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if payload.network_key.len() != 64
        || !payload.network_key.chars().all(|c| c.is_ascii_hexdigit())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid key format (must be 64 hex chars)".to_string(),
        ));
    }

    let env_path = ".env";

    let contents = fs::read_to_string(env_path).await.unwrap_or_default();

    let mut new_lines = Vec::new();
    let mut key_found = false;

    for line in contents.lines() {
        if line.starts_with("SYNC_NETWORK_KEY=") {
            new_lines.push(format!("SYNC_NETWORK_KEY={}", payload.network_key));
            key_found = true;
        } else {
            new_lines.push(line.to_string());
        }
    }

    if !key_found {
        new_lines.push(format!("SYNC_NETWORK_KEY={}", payload.network_key));
    }

    let mut output = new_lines.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }

    if let Err(e) = fs::write(env_path, output).await {
        error!("Failed to write .env: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to write configuration file".to_string(),
        ));
    }

    info!("Configuration updated: SYNC_NETWORK_KEY saved.");

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Key saved. Please restart the server."
    })))
}
