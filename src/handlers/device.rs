use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    body::Body,
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::AppState;
use crate::models::registered_device;
use crate::utils::auth;
use crate::utils::identity;

// ============================================================
// Request / Response types
// ============================================================

#[derive(Deserialize)]
pub struct DeviceRegisterRequest {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "deviceName")]
    pub device_name: Option<String>,
    #[serde(rename = "devicePublicKey")]
    pub device_public_key: String,
    pub signature: String,
    #[serde(rename = "inviteToken")]
    pub invite_token: Option<String>,
}

#[derive(Serialize)]
pub struct DeviceRegisterResponse {
    pub success: bool,
    pub status: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub token: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enc_key: Option<String>,
}

#[derive(Deserialize)]
pub struct PairingQrQuery {
    #[serde(rename = "type")]
    pub qr_type: Option<String>,
}

#[derive(Deserialize)]
pub struct ListDevicesQuery {
    pub include_deleted: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(Deserialize)]
pub struct UpdateHomeRequest {
    #[serde(rename = "homeInstanceId")]
    pub home_instance_id: String,
}

// ============================================================
// POST /api/internal/register-device (PUBLIC — no JWT)
// ============================================================

pub async fn register_device(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DeviceRegisterRequest>,
) -> Result<Json<DeviceRegisterResponse>, (StatusCode, String)> {
    // 1. Validate required fields
    if body.device_id.is_empty() || body.device_public_key.is_empty() || body.signature.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Missing required fields".into()));
    }

    // 2. Verify Ed25519 signature
    let message = format!(
        "{{\"deviceId\":\"{}\",\"devicePublicKey\":\"{}\"}}",
        body.device_id, body.device_public_key
    );

    let valid = identity::verify_signature(&body.device_public_key, &message, &body.signature)
        .map_err(|e| (StatusCode::FORBIDDEN, format!("Signature verification failed: {}", e)))?;

    if !valid {
        return Err((StatusCode::FORBIDDEN, "Invalid signature".into()));
    }

    // 3. Determine initial status
    let mut final_status = "pending".to_string();

    if let Some(ref invite_token) = body.invite_token {
        if !invite_token.is_empty() {
            if let Ok(true) = auth::validate_invite_token(invite_token, &state.config.jwt_secret) {
                final_status = "active".to_string();
            }
        }
    }

    // 4. Upsert device in DB (find even soft-deleted)
    let existing = registered_device::Entity::find_by_id(&body.device_id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(existing_device) = existing {
        // Read current state before converting to ActiveModel
        let was_deleted = existing_device.deleted_at.is_some();
        let current_status = existing_device.status.clone().unwrap_or_default();

        let mut active: registered_device::ActiveModel = existing_device.into();

        active.name = Set(body.device_name.clone());
        active.public_key = Set(Some(body.device_public_key.clone()));
        active.last_seen_at = Set(Some(Utc::now()));
        active.updated_at = Set(Some(Utc::now()));
        active.home_instance_id = Set(Some(state.config.instance_id.clone()));

        if was_deleted {
            // Restore soft-deleted device
            active.deleted_at = Set(None);
            active.status = Set(Some(final_status.clone()));
        } else {
            // Only auto-approve pending devices with valid invite token
            if current_status == "pending" && final_status == "active" {
                active.status = Set(Some("active".to_string()));
            } else {
                // Keep current status (don't unblock blocked devices)
                final_status = current_status;
            }
        }

        active.update(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    } else {
        // Create new device
        let new_device = registered_device::ActiveModel {
            device_id: Set(body.device_id.clone()),
            name: Set(body.device_name.clone()),
            public_key: Set(Some(body.device_public_key.clone())),
            status: Set(Some(final_status.clone())),
            home_instance_id: Set(Some(state.config.instance_id.clone())),
            last_seen_at: Set(Some(Utc::now())),
            created_at: Set(Some(Utc::now())),
            updated_at: Set(Some(Utc::now())),
            deleted_at: Set(None),
        };

        new_device
            .insert(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    // 5. Generate JWT if active
    let mut access_token = String::new();
    if final_status == "active" {
        if let Ok(token) = auth::generate_device_token(&body.device_id, &state.config.jwt_secret) {
            access_token = token;
        }
    }

    // 6. Include enc_key for active devices
    let enc_key = if final_status == "active" {
        std::env::var("ENC_KEY").ok().filter(|k| !k.is_empty())
    } else {
        None
    };

    tracing::info!(
        "Device registration: {} ({}) -> status={}",
        body.device_id,
        body.device_name.as_deref().unwrap_or("unnamed"),
        final_status
    );

    Ok(Json(DeviceRegisterResponse {
        success: true,
        status: final_status,
        token: access_token,
        message: "Device handshake complete".into(),
        enc_key,
    }))
}

// ============================================================
// GET /api/internal/pairing-qr (JWT protected)
// ============================================================

pub async fn generate_pairing_qr(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PairingQrQuery>,
) -> Result<Response, (StatusCode, String)> {
    let identity = &state.server_identity;

    // 1. Compact UUID (remove dashes, uppercase)
    let compact_uuid = identity
        .instance_id
        .replace('-', "")
        .to_uppercase();

    // 2. Public key hex (uppercase)
    let pub_key_hex = identity
        .public_key_hex()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // 3. Build connection candidates
    let mut candidates = Vec::new();
    let port = state.config.port;

    // Add local IPs
    if let Ok(local_ip) = local_ip_address::local_ip() {
        candidates.push(format!("http://{}:{}/E", local_ip, port));
    }

    // Add all local IPs from network interfaces
    if let Ok(ifaces) = local_ip_address::list_afinet_netifas() {
        for (_, ip) in &ifaces {
            if ip.is_ipv4() && !ip.is_loopback() {
                let url = format!("http://{}:{}/E", ip, port);
                if !candidates.contains(&url) {
                    candidates.push(url);
                }
            }
        }
    }

    // Add global URL
    if !state.config.base_url.is_empty() {
        let mut global = state.config.base_url.clone();
        if !global.ends_with('/') {
            global.push('/');
        }
        candidates.push(global);
    }

    let connection_string = candidates.join(",").to_uppercase();

    // 4. Handle VIP/invite token
    let invite_suffix = if params.qr_type.as_deref() == Some("vip") {
        match auth::generate_invite_token(&state.config.jwt_secret) {
            Ok(token) => format!("${}", token),
            Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
        }
    } else {
        String::new()
    };

    // 5. Build QR string: ECK$2$UUID$KEY$URLS[$TOKEN]
    let qr_string = format!(
        "ECK$2${}${}${}{}",
        compact_uuid, pub_key_hex, connection_string, invite_suffix
    );

    // 6. Generate QR code PNG
    let qr = qrcode::QrCode::new(qr_string.as_bytes())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("QR generation failed: {}", e)))?;

    let image = qr.render::<image::Luma<u8>>().quiet_zone(true).max_dimensions(512, 512).build();

    let mut png_data = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
    image::ImageEncoder::write_image(
        encoder,
        image.as_raw(),
        image.width(),
        image.height(),
        image::ExtendedColorType::L8,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("PNG encoding failed: {}", e)))?;

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "image/png")
        .body(Body::from(png_data))
        .unwrap())
}

// ============================================================
// GET /api/admin/devices
// ============================================================

pub async fn list_devices(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListDevicesQuery>,
) -> Result<Json<Vec<registered_device::Model>>, (StatusCode, String)> {
    let include_deleted = params.include_deleted.as_deref() == Some("true");

    let mut query = registered_device::Entity::find();

    if !include_deleted {
        query = query.filter(registered_device::Column::DeletedAt.is_null());
    }

    // Order: pending first, then active, then blocked
    query = query.order_by_asc(sea_orm::sea_query::SimpleExpr::Custom(
        "CASE WHEN status = 'pending' THEN 1 WHEN status = 'active' THEN 2 ELSE 3 END".into(),
    ));

    let devices = query
        .all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(devices))
}

// ============================================================
// PUT /api/admin/devices/:id/status
// ============================================================

pub async fn update_device_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateStatusRequest>,
) -> Result<Json<registered_device::Model>, (StatusCode, String)> {
    let valid_statuses = ["active", "pending", "blocked"];
    if !valid_statuses.contains(&body.status.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "Invalid status".into()));
    }

    let device = registered_device::Entity::find_by_id(&id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Device not found".into()))?;

    let mut active: registered_device::ActiveModel = device.into();
    active.status = Set(Some(body.status));
    active.updated_at = Set(Some(Utc::now()));

    let updated = active
        .update(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(updated))
}

// ============================================================
// PUT /api/admin/devices/:id/home
// ============================================================

pub async fn update_device_home(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateHomeRequest>,
) -> Result<Json<registered_device::Model>, (StatusCode, String)> {
    let device = registered_device::Entity::find_by_id(&id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Device not found".into()))?;

    let mut active: registered_device::ActiveModel = device.into();
    active.home_instance_id = Set(Some(body.home_instance_id));
    active.updated_at = Set(Some(Utc::now()));

    let updated = active
        .update(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(updated))
}

// ============================================================
// DELETE /api/admin/devices/:id (soft delete)
// ============================================================

pub async fn delete_device(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let device = registered_device::Entity::find_by_id(&id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Device not found".into()))?;

    let mut active: registered_device::ActiveModel = device.into();
    active.deleted_at = Set(Some(Utc::now()));
    active.updated_at = Set(Some(Utc::now()));

    active
        .update(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Device deleted successfully (soft deleted for sync)",
        "id": id
    })))
}

// ============================================================
// POST /api/admin/devices/:id/restore
// ============================================================

pub async fn restore_device(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Need to find even deleted devices
    let device = registered_device::Entity::find_by_id(&id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Device not found".into()))?;

    if device.deleted_at.is_none() {
        return Err((StatusCode::BAD_REQUEST, "Device is not deleted".into()));
    }

    let mut active: registered_device::ActiveModel = device.into();
    active.deleted_at = Set(None);
    active.updated_at = Set(Some(Utc::now()));

    active
        .update(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Device restored successfully (will sync to mesh)",
        "id": id
    })))
}
