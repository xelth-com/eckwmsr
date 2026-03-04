use axum::{extract::State, Json};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use crate::{
    ai::prompts::AGENT_SYSTEM_PROMPT,
    db::AppState,
    models::{location, product, product_alias},
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
    #[serde(rename = "ai_interaction", skip_serializing_if = "Option::is_none")]
    pub ai_interaction: Option<serde_json::Value>,
    #[serde(rename = "msgId", skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<String>,
}

/// POST /api/scan — universal barcode scanner endpoint.
/// Decodes Smart Codes (i/b/p/l), performs DB lookups, falls back to AI.
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
            ai_interaction: None,
            msg_id: payload.msg_id,
        });
    }

    info!("[SCAN] Received barcode: {}", barcode);

    // Check for SmartTag entity prefixes (company-uuid, person-uuid, opp-uuid)
    if let Some(resp) = try_twenty_lookup(&barcode, &state, &payload).await {
        return Json(resp);
    }

    let prefix = barcode.chars().next().unwrap_or('_');

    let mut resp = ScanResponse {
        r#type: "unknown".into(),
        message: "Unknown or legacy barcode".into(),
        action: "error".into(),
        data: None,
        ai_interaction: None,
        msg_id: payload.msg_id.clone(),
    };

    match prefix {
        'p' | 'P' => {
            if let Ok(data) = decode_smart_place(&barcode.to_lowercase()) {
                if let Ok(Some(loc)) = location::Entity::find_by_id(data.location_id)
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
            // 1. Check memory (product_alias)
            if let Ok(Some(alias)) = product_alias::Entity::find()
                .filter(product_alias::Column::ExternalCode.eq(&barcode))
                .one(&state.db)
                .await
            {
                resp.r#type = "alias".into();
                resp.action = "found".into();
                resp.message = format!("Alias for {}", alias.internal_id);
                resp.data = Some(serde_json::to_value(&alias).unwrap_or_default());
                return Json(resp);
            }

            // 2. Legacy DB lookup (product by barcode or default_code)
            if let Ok(Some(prod)) = product::Entity::find()
                .filter(
                    Condition::any()
                        .add(product::Column::Barcode.eq(&barcode))
                        .add(product::Column::DefaultCode.eq(&barcode)),
                )
                .one(&state.db)
                .await
            {
                resp.r#type = "product".into();
                resp.action = "found".into();
                resp.message = prod.name.clone();
                resp.data = Some(serde_json::to_value(&prod).unwrap_or_default());
                return Json(resp);
            }

            // 3. Fallback to AI for truly unknown barcodes
            if let Some(ai) = &state.ai_client {
                let full_prompt = format!(
                    "Worker scanned unknown code: '{}'. Analyze it.",
                    barcode
                );

                match ai.generate_content(AGENT_SYSTEM_PROMPT, &full_prompt).await {
                    Ok(ai_response_str) => {
                        let clean_json = crate::utils::json::sanitize_json(&ai_response_str);
                        if let Ok(interaction) =
                            serde_json::from_str::<serde_json::Value>(&clean_json)
                        {
                            resp.r#type = "ai_analysis".into();
                            resp.action = "interaction".into();
                            resp.message = "AI Analysis".into();
                            resp.ai_interaction = Some(interaction);
                            return Json(resp);
                        } else {
                            error!("AI JSON Parse Error: {}", clean_json);
                        }
                    }
                    Err(e) => {
                        error!("AI Gen Error: {}", e);
                    }
                }
            }

            resp.message = "Legacy barcode not found".into();
        }
    }

    Json(resp)
}

/// Try to route SmartTag entity prefixes (company-, person-, opp-) to Twenty CRM.
async fn try_twenty_lookup(
    barcode: &str,
    state: &Arc<AppState>,
    payload: &ScanRequest,
) -> Option<ScanResponse> {
    let (entity_type, uuid) = if let Some(uuid) = barcode.strip_prefix("company-") {
        ("company", uuid)
    } else if let Some(uuid) = barcode.strip_prefix("person-") {
        ("person", uuid)
    } else if let Some(uuid) = barcode.strip_prefix("opp-") {
        ("opportunity", uuid)
    } else {
        return None;
    };

    let client = match &state.twenty_client {
        Some(c) => c,
        None => {
            return Some(ScanResponse {
                r#type: entity_type.into(),
                message: "Twenty CRM integration not configured".into(),
                action: "error".into(),
                data: None,
                ai_interaction: None,
                msg_id: payload.msg_id.clone(),
            });
        }
    };

    let result = match entity_type {
        "company" => client.get_company(uuid).await,
        "person" => client.get_person(uuid).await,
        "opportunity" => client.get_opportunity(uuid).await,
        _ => unreachable!(),
    };

    match result {
        Ok(data) => {
            let name = data["name"]
                .as_str()
                .or_else(|| {
                    // People may have firstName + lastName instead of name
                    let first = data["name"]["firstName"].as_str().unwrap_or("");
                    let last = data["name"]["lastName"].as_str().unwrap_or("");
                    if first.is_empty() && last.is_empty() {
                        None
                    } else {
                        // Return None here — we'll build it below
                        None
                    }
                })
                .unwrap_or(entity_type);

            // Build display name for people (firstName + lastName)
            let display_name = if entity_type == "person" {
                let first = data["name"]["firstName"].as_str().unwrap_or("");
                let last = data["name"]["lastName"].as_str().unwrap_or("");
                if !first.is_empty() || !last.is_empty() {
                    format!("{} {}", first, last).trim().to_string()
                } else {
                    name.to_string()
                }
            } else {
                name.to_string()
            };

            Some(ScanResponse {
                r#type: entity_type.into(),
                message: display_name,
                action: "found".into(),
                data: Some(data),
                ai_interaction: None,
                msg_id: payload.msg_id.clone(),
            })
        }
        Err(e) => {
            error!("[SCAN] Twenty {} lookup failed: {}", entity_type, e);
            Some(ScanResponse {
                r#type: entity_type.into(),
                message: format!("{} not found", entity_type),
                action: "not_found".into(),
                data: None,
                ai_interaction: None,
                msg_id: payload.msg_id.clone(),
            })
        }
    }
}
