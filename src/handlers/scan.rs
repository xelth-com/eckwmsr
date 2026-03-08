use axum::{extract::State, Json};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use uuid::Uuid;

use crate::{
    ai::prompts::AGENT_SYSTEM_PROMPT,
    db::AppState,
    models::{item, location, order, product, product_alias},
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
    /// Trust level: "full" for V2 SmartTags, "soft" for legacy single-match, absent if N/A
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust: Option<String>,
}

/// POST /api/scan — universal barcode scanner endpoint.
/// Decodes Smart Codes (i/b/p/l), performs DB lookups, falls back to AI.
/// Supports trust levels: V2 SmartTags = full trust, legacy matches = soft trust,
/// multiple matches = ambiguous collision requiring user resolution.
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
            trust: None,
        });
    }

    info!("[SCAN] Received barcode: {}", barcode);

    // Check for SmartTag entity prefixes (company-uuid, person-uuid, opp-uuid)
    if let Some(resp) = try_twenty_lookup(&barcode, &state, &payload).await {
        return Json(resp);
    }

    // V2 Binary SmartTags: single-char prefix + '-' + UUID (e.g. i-UUID, p-UUID, b-UUID)
    // These are Trust 100% (full) — cryptographically bound to a single entity.
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
                        trust: Some("full".into()),
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
                            resp.trust = None;
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
                        trust: Some("full".into()),
                    };
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
                            resp.trust = None;
                        }
                    }
                    return Json(resp);
                }
                'b' | 'B' => {
                    return Json(ScanResponse {
                        r#type: "box".into(),
                        message: format!("Box {}", native_id),
                        action: "decoded".into(),
                        data: Some(serde_json::json!({ "id": native_id.to_string() })),
                        ai_interaction: None,
                        msg_id: payload.msg_id.clone(),
                        trust: Some("full".into()),
                    });
                }
                _ => {} // fall through to legacy
            }
        }
    }

    let prefix = barcode.chars().next().unwrap_or('_');

    // Smart label codes
    if prefix == 'l' || prefix == 'L' {
        if let Ok(data) = decode_smart_label(&barcode) {
            return Json(ScanResponse {
                r#type: "label".into(),
                message: "Smart Label scanned".into(),
                action: "decoded".into(),
                data: Some(serde_json::json!({
                    "type": data.label_type,
                    "payload": data.payload,
                    "date": data.date.to_rfc3339()
                })),
                ai_interaction: None,
                msg_id: payload.msg_id.clone(),
                trust: Some("full".into()),
            });
        }
    }

    // ── Legacy / generic barcode: parallel search with trust levels ──

    // 1. Check product_alias (memory) — exact match, high trust
    if let Ok(Some(alias)) = product_alias::Entity::find()
        .filter(product_alias::Column::ExternalCode.eq(&barcode))
        .one(&state.db)
        .await
    {
        return Json(ScanResponse {
            r#type: "alias".into(),
            message: format!("Alias for {}", alias.internal_id),
            action: "found".into(),
            data: Some(serde_json::to_value(&alias).unwrap_or_default()),
            ai_interaction: None,
            msg_id: payload.msg_id.clone(),
            trust: Some("soft".into()),
        });
    }

    // 2. Parallel search: orders + items + products
    let mut candidates: Vec<serde_json::Value> = Vec::new();

    // Search orders by order_number or serial_number
    if let Ok(orders) = order::Entity::find()
        .filter(
            Condition::any()
                .add(order::Column::OrderNumber.eq(&barcode))
                .add(order::Column::SerialNumber.eq(&barcode)),
        )
        .all(&state.db)
        .await
    {
        for o in orders {
            candidates.push(serde_json::json!({
                "id": o.id.to_string(),
                "type": "order",
                "title": format!("Order {}", o.order_number),
                "subtitle": format!("{} — {}", o.customer_name, o.status),
                "date": o.created_at.to_rfc3339(),
                "barcode": o.order_number,
            }));
        }
    }

    // Search items by primary_barcode
    if let Ok(items) = item::Entity::find()
        .filter(
            Condition::any()
                .add(item::Column::PrimaryBarcode.eq(&barcode)),
        )
        .filter(item::Column::DeletedAt.is_null())
        .all(&state.db)
        .await
    {
        for it in items {
            candidates.push(serde_json::json!({
                "id": it.id.to_string(),
                "type": "item",
                "title": format!("Item {}", it.name.as_deref().unwrap_or(&it.primary_barcode)),
                "subtitle": it.primary_barcode,
                "date": it.created_at.to_rfc3339(),
                "barcode": it.primary_barcode,
            }));
        }
    }

    // Search products by barcode or default_code
    if let Ok(Some(prod)) = product::Entity::find()
        .filter(
            Condition::any()
                .add(product::Column::Barcode.eq(&barcode))
                .add(product::Column::DefaultCode.eq(&barcode)),
        )
        .one(&state.db)
        .await
    {
        candidates.push(serde_json::json!({
            "id": prod.id,
            "type": "product",
            "title": prod.name,
            "subtitle": prod.default_code,
            "date": null,
            "barcode": prod.barcode,
        }));
    }

    // 3. Resolve candidates
    match candidates.len() {
        0 => {
            // No matches — fall back to AI
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
                            return Json(ScanResponse {
                                r#type: "ai_analysis".into(),
                                message: "AI Analysis".into(),
                                action: "interaction".into(),
                                data: None,
                                ai_interaction: Some(interaction),
                                msg_id: payload.msg_id.clone(),
                                trust: None,
                            });
                        } else {
                            error!("AI JSON Parse Error: {}", clean_json);
                        }
                    }
                    Err(e) => {
                        error!("AI Gen Error: {}", e);
                    }
                }
            }

            Json(ScanResponse {
                r#type: "unknown".into(),
                message: "Legacy barcode not found".into(),
                action: "error".into(),
                data: None,
                ai_interaction: None,
                msg_id: payload.msg_id.clone(),
                trust: None,
            })
        }
        1 => {
            // Single match — soft trust
            let c = &candidates[0];
            let c_type = c["type"].as_str().unwrap_or("unknown");
            let c_id = c["id"].as_str().unwrap_or("");
            let c_title = c["title"].as_str().unwrap_or("Unknown");

            // Build data with the full entity if it's an order or item
            let data = match c_type {
                "order" => {
                    if let Ok(uuid) = Uuid::parse_str(c_id) {
                        order::Entity::find_by_id(uuid)
                            .one(&state.db)
                            .await
                            .ok()
                            .flatten()
                            .and_then(|o| serde_json::to_value(&o).ok())
                    } else {
                        Some(c.clone())
                    }
                }
                "item" => {
                    if let Ok(uuid) = Uuid::parse_str(c_id) {
                        item::Entity::find_by_id(uuid)
                            .one(&state.db)
                            .await
                            .ok()
                            .flatten()
                            .and_then(|i| serde_json::to_value(&i).ok())
                    } else {
                        Some(c.clone())
                    }
                }
                _ => Some(c.clone()),
            };

            Json(ScanResponse {
                r#type: c_type.into(),
                message: c_title.into(),
                action: "found".into(),
                data,
                ai_interaction: None,
                msg_id: payload.msg_id.clone(),
                trust: Some("soft".into()),
            })
        }
        _ => {
            // Multiple matches — ambiguous collision
            info!("[SCAN] Ambiguous: {} candidates for '{}'", candidates.len(), barcode);
            Json(ScanResponse {
                r#type: "ambiguous".into(),
                message: "Multiple matches found".into(),
                action: "collision".into(),
                data: Some(serde_json::json!({ "candidates": candidates })),
                ai_interaction: None,
                msg_id: payload.msg_id.clone(),
                trust: None,
            })
        }
    }
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
                trust: None,
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
                    let first = data["name"]["firstName"].as_str().unwrap_or("");
                    let last = data["name"]["lastName"].as_str().unwrap_or("");
                    if first.is_empty() && last.is_empty() {
                        None
                    } else {
                        None
                    }
                })
                .unwrap_or(entity_type);

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
                trust: Some("full".into()),
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
                trust: None,
            })
        }
    }
}
