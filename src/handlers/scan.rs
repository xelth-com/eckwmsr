use axum::{extract::State, Json};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::{
    db::AppState,
    models::location,
    utils::smart_code::{decode_smart_box, decode_smart_item, decode_smart_label, decode_smart_place},
};

#[derive(Deserialize)]
pub struct ScanRequest {
    pub barcode: String,
    #[serde(rename = "msgId")]
    pub msg_id: Option<String>,
    #[serde(rename = "deviceId")]
    pub device_id: Option<String>,
}

#[derive(Serialize)]
pub struct ScanResponse {
    pub r#type: String,
    pub message: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(rename = "msgId", skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<String>,
}

/// POST /api/scan — universal barcode scanner endpoint.
/// Decodes Smart Codes (i/b/p/l) and performs DB lookups.
/// Mirrors Go's `handleScan` from `internal/handlers/scan.go`.
pub async fn handle_scan(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ScanRequest>,
) -> Json<ScanResponse> {
    let barcode = payload.barcode.trim().to_string();

    if barcode.is_empty() {
        return Json(ScanResponse {
            r#type: "error".into(),
            message: "Empty barcode".into(),
            action: "error".into(),
            data: None,
            msg_id: payload.msg_id,
        });
    }

    let prefix = barcode.chars().next().unwrap_or('_');
    info!("[SCAN] Received barcode: {} (prefix: {})", barcode, prefix);

    let mut resp = ScanResponse {
        r#type: "unknown".into(),
        message: "Unknown or legacy barcode".into(),
        action: "error".into(),
        data: None,
        msg_id: payload.msg_id.clone(),
    };

    match prefix {
        'p' | 'P' => {
            if let Ok(data) = decode_smart_place(&barcode.to_lowercase()) {
                if let Ok(Some(loc)) =
                    location::Entity::find_by_id(data.location_id)
                        .one(&state.db)
                        .await
                {
                    resp.r#type = "place".into();
                    resp.action = "found".into();
                    resp.message = loc.complete_name.clone();
                    resp.data = Some(serde_json::to_value(&loc).unwrap_or_default());
                } else {
                    resp.r#type = "place".into();
                    resp.action = "not_found".into();
                    resp.message = "Location not found in DB".into();
                }
            }
        }
        'i' | 'I' => {
            if let Ok(data) = decode_smart_item(&barcode) {
                resp.r#type = "item".into();
                resp.action = "decoded".into();
                resp.message = "Item scanned".into();
                resp.data = Some(serde_json::json!({
                    "serial": data.serial,
                    "ref_id": data.ref_id
                }));
            }
        }
        'b' | 'B' => {
            if let Ok(data) = decode_smart_box(&barcode) {
                resp.r#type = "box".into();
                resp.action = "decoded".into();
                resp.message = "Box scanned".into();
                resp.data = Some(serde_json::json!({
                    "length": data.length,
                    "width": data.width,
                    "height": data.height,
                    "weight": data.weight,
                    "type": data.pkg_type,
                    "serial": data.serial
                }));
            }
        }
        'l' | 'L' => {
            if let Ok(data) = decode_smart_label(&barcode) {
                resp.r#type = "label".into();
                resp.action = "decoded".into();
                resp.message = "Smart Label scanned".into();
                resp.data = Some(serde_json::json!({
                    "type": data.label_type,
                    "payload": data.payload,
                    "date": data.date.to_rfc3339()
                }));
            }
        }
        _ => {
            // Legacy barcode fallback — search product by EAN/barcode
            resp.message = "Legacy barcode format".into();
        }
    }

    Json(resp)
}
