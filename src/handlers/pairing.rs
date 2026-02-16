use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::AppState;
use crate::models::mesh_node;
use crate::services::pairing::PairingService;

// -- Request / Response types --

#[derive(Serialize)]
pub struct GenerateCodeResponse {
    pub code: String,
    pub expires_in: u64,
    pub mesh_id: String,
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
    pub mesh_id: String,
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

#[derive(Deserialize)]
pub struct ApproveRequest {
    pub code: String,
    pub remote_instance_id: String,
    pub remote_instance_name: String,
}

#[derive(Deserialize)]
pub struct FinalizeRequest {
    pub code: String,
}

#[derive(Serialize)]
pub struct FinalizeResponse {
    pub status: String,
    pub network_key: Option<String>,
    pub host_instance_id: Option<String>,
    pub host_base_url: Option<String>,
}

// -- Helpers --

fn make_pairing_service(state: &AppState) -> PairingService {
    PairingService::new(
        state.config.instance_id.clone(),
        state.config.instance_id.clone(), // instance_name = instance_id for now
        state.config.sync_relay_url.clone(),
        state.sync_engine.relay_client(),
    )
}

// -- Endpoints --

/// POST /api/pairing/host — generates a magic code and publishes a pairing offer
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
        mesh_id: state.config.mesh_id.clone(),
    }))
}

/// POST /api/pairing/connect — client enters a code and finds the offer
pub async fn join_pairing(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ConnectRequest>,
) -> Result<Json<ConnectResponse>, (StatusCode, String)> {
    let svc = make_pairing_service(&state);

    let offer = svc
        .find_offer(&payload.code)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    svc.send_response(&payload.code)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ConnectResponse {
        status: "paired".to_string(),
        remote_instance_id: offer.instance_id,
        remote_instance_name: offer.instance_name,
        mesh_id: state.config.mesh_id.clone(),
    }))
}

/// POST /api/pairing/check — host polls to see if a client responded
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

/// POST /api/pairing/approve — host approves a discovered client, saves it as a mesh node
pub async fn approve_pairing(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ApproveRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let node = mesh_node::ActiveModel {
        instance_id: Set(payload.remote_instance_id.clone()),
        name: Set(payload.remote_instance_name.clone()),
        base_url: Set(String::new()),
        role: Set("peer".to_string()),
        status: Set("active".to_string()),
        last_seen: Set(Utc::now()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    node.insert(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!(
        "Pairing approved: {} -> {}",
        state.config.instance_id,
        payload.remote_instance_id
    );

    Ok(Json(serde_json::json!({
        "status": "approved",
        "mesh_id": state.config.mesh_id
    })))
}

/// POST /api/pairing/finalize — client finalizes pairing, saves the host as a mesh node
pub async fn finalize_pairing(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<FinalizeRequest>,
) -> Result<Json<FinalizeResponse>, (StatusCode, String)> {
    let svc = make_pairing_service(&state);

    // Try to find the offer again to get the host's identity
    let offer = svc.find_offer(&payload.code).await.ok();

    if let Some(ref offer) = offer {
        let node = mesh_node::ActiveModel {
            instance_id: Set(offer.instance_id.clone()),
            name: Set(offer.instance_name.clone()),
            base_url: Set(offer.relay_url.clone()),
            role: Set("master".to_string()),
            status: Set("active".to_string()),
            last_seen: Set(Utc::now()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let _ = node.insert(&state.db).await;

        tracing::info!(
            "Pairing finalized: {} saved host {}",
            state.config.instance_id,
            offer.instance_id
        );
    }

    Ok(Json(FinalizeResponse {
        status: if offer.is_some() {
            "finalized".to_string()
        } else {
            "waiting".to_string()
        },
        network_key: None, // Key exchange happens separately
        host_instance_id: offer.as_ref().map(|o| o.instance_id.clone()),
        host_base_url: offer.as_ref().map(|o| o.relay_url.clone()),
    }))
}
