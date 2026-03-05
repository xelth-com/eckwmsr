use axum::{extract::State, Json};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use uuid::Uuid;

use crate::{
    ai::prompts::AGENT_SYSTEM_PROMPT,
    db::AppState,
    models::{item, location, product, product_alias},
    utils::smart_code::decode_smart_label,
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

    // V2 Binary SmartTags: single-char prefix + '-' + UUID (e.g. i-UUID, p-UUID, b-UUID)
    if barcode.len() >= 38 && barcode.as_bytes().get(1) == Some(&b'-') {
        if let Ok(native_id) = Uuid::parse_str(&barcode[2..]) {
            let v2_prefix = barcode.chars().next().unwrap_or('_');
            match v2_prefix {
                'i' | 'I' => {
                    let mut resp = ScanResponse {
                        r#type: "item".into(),
                        message: "Item found".into(),
                        action: "found".into(),
                        data: None,
                        ai_interaction: None,
                        msg_id: payload.msg_id.clone(),
                    };
                    match item::Entity::find_by_id(native_id)
                        .filter(item::Column::DeletedAt.is_null())
                        .one(&state.db)
                        .await
                    {
                        Ok(Some(found)) => {
                            if let Some(ref name) = found.name {
                                resp.message = name.clone();
                            }
                            resp.data = Some(serde_json::to_value(&found).unwrap_or_default());
                        }
                        _ => {
                            resp.action = "not_found".into();
                            resp.message = format!("Item {} not in DB", native_id);
                        }
                    }
                    return Json(resp);
                }
                'p' | 'P' => {
                    let mut resp = ScanResponse {
                        r#type: "place".into(),
                        message: "Location found".into(),
                        action: "found".into(),
                        data: None,
                        ai_interaction: None,
                        msg_id: payload.msg_id.clone(),
                    };
                    // V2 place tags use UUID — try location lookup by barcode string
                    match location::Entity::find()
                        .filter(location::Column::Barcode.eq(&barcode))
                        .one(&state.db)
                        .await
                    {
                        Ok(Some(loc)) => {
                            resp.message = loc.complete_name.clone();
                            resp.data = Some(serde_json::to_value(&loc).unwrap_or_default());
                        }
                        _ => {
                            resp.action = "not_found".into();
                            resp.message = format!("Location {} not in DB", native_id);
                        }
                    }
                    return Json(resp);
                }
                'b' | 'B' => {
                    // V2 box tag — return decoded info with native UUID
                    return Json(ScanResponse {
                        r#type: "box".into(),
                        message: format!("Box {}", native_id),
                        action: "decoded".into(),
                        data: Some(serde_json::json!({ "id": native_id.to_string() })),
                        ai_interaction: None,
                        msg_id: payload.msg_id.clone(),
                    });
                }
                _ => {} // fall through to legacy
            }
        }
    }

    let prefix = barcode.chars().next().unwrap_or('_');

    let mut resp = ScanResponse {
        r#type: "unknown".into(),
        message: "Unknown barcode".into(),
        action: "error".into(),
        data: None,
        ai_interaction: None,
        msg_id: payload.msg_id.clone(),
    };

    match prefix {
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
