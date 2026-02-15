use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::AppState;
use crate::services::pairing::PairingService;

// ── Request / Response types ──────────────────────────────────────

#[derive(Serialize)]
pub struct GenerateCodeResponse {
    pub code: String,
    pub expires_in: u64,
}

#[derive(Deserialize)]
pub struct ConnectRequest {
    pub code: String,
}

#[derive(Serialize)]
pub struct ConnectResponse {
    pub status: String,
    pub remote_instance_id: String,
    pub remote_instance_name: String,
}

#[derive(Deserialize)]
pub struct CheckRequest {
    pub code: String,
}

#[derive(Serialize)]
pub struct CheckResponse {
    pub found: bool,
    pub remote_instance_id: Option<String>,
    pub remote_instance_name: Option<String>,
}

// ── Helpers ───────────────────────────────────────────────────────

fn make_pairing_service(state: &AppState) -> PairingService {
    PairingService::new(
        state.config.instance_id.clone(),
        state.config.instance_id.clone(), // instance_name = instance_id for now
        state.config.sync_relay_url.clone(),
        state.sync_engine.relay_client(),
    )
}

// ── Endpoints ─────────────────────────────────────────────────────

/// POST /api/pairing/host
/// Generates a magic code and publishes a pairing offer to the relay.
/// Returns the code for the user to share with the other server.
pub async fn host_pairing(
    State(state): State<Arc<AppState>>,
) -> Result<Json<GenerateCodeResponse>, (StatusCode, String)> {
    let svc = make_pairing_service(&state);
    let code = PairingService::generate_code();

    svc.publish_offer(&code)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(GenerateCodeResponse {
        code,
        expires_in: 300,
    }))
}

/// POST /api/pairing/connect
/// Client enters a magic code. Finds the offer on the relay,
/// then sends a response back so the Host knows we want to connect.
pub async fn join_pairing(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ConnectRequest>,
) -> Result<Json<ConnectResponse>, (StatusCode, String)> {
    let svc = make_pairing_service(&state);

    // Step 1: Find the offer
    let offer = svc
        .find_offer(&payload.code)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    // Step 2: Send our response back through the relay
    svc.send_response(&payload.code)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ConnectResponse {
        status: "paired".to_string(),
        remote_instance_id: offer.instance_id,
        remote_instance_name: offer.instance_name,
    }))
}

/// POST /api/pairing/check
/// Host polls this endpoint to see if anyone has responded to their code.
pub async fn check_pairing(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CheckRequest>,
) -> Result<Json<CheckResponse>, (StatusCode, String)> {
    let svc = make_pairing_service(&state);

    let response = svc
        .check_response(&payload.code)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match response {
        Some(resp) => Ok(Json(CheckResponse {
            found: true,
            remote_instance_id: Some(resp.instance_id),
            remote_instance_name: Some(resp.instance_name),
        })),
        None => Ok(Json(CheckResponse {
            found: false,
            remote_instance_id: None,
            remote_instance_name: None,
        })),
    }
}
