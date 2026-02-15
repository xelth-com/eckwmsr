use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    db::AppState,
    models::{document, inventory_discrepancy, location, product, quant},
    services::repair::RepairService,
};

#[derive(Deserialize)]
pub struct RepairEventRequest {
    pub source_device_id: String,
    pub target_device_id: String,
    pub event_type: String,
    pub data: String,
    pub acting_user_id: Option<String>,
    pub owner_user_id: Option<String>,
}

#[derive(Serialize)]
pub struct DiscrepancyResponse {
    pub barcode: String,
    pub product_name: String,
    pub product_code: String,
    pub expected_qty: f64,
    pub counted_qty: f64,
    pub delta: f64,
    pub location: String,
}

/// POST /api/repair/event — receives repair workflow events and stores as documents.
/// Mirrors Go's `handleRepairEvent` from `internal/handlers/repair.go`.
pub async fn handle_repair_event(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RepairEventRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if payload.target_device_id.is_empty() || payload.event_type.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "target_device_id and event_type are required".to_string(),
        ));
    }

    let doc_type = match payload.target_device_id.as_str() {
        "RESTOCK" => "restock_order",
        "INVENTORY" => "inventory_count",
        _ => "repair_log",
    };

    let mut json_payload = serde_json::json!({
        "target_device_id": payload.target_device_id,
        "event_type": payload.event_type,
        "data": payload.data,
        "timestamp": Utc::now().to_rfc3339(),
    });

    if let Some(ref actor) = payload.acting_user_id {
        json_payload["acting_user_id"] = serde_json::Value::String(actor.clone());
    }
    if let Some(ref owner) = payload.owner_user_id {
        json_payload["owner_user_id"] = serde_json::Value::String(owner.clone());
    }

    let doc_id = Uuid::new_v4();
    let doc = document::ActiveModel {
        id: Set(doc_id),
        r#type: Set(doc_type.to_string()),
        status: Set("processed".to_string()),
        device_id: Set(payload.source_device_id.clone()),
        user_id: Set(payload.acting_user_id.clone().unwrap_or_default()),
        payload: Set(json_payload),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    if let Err(e) = doc.insert(&state.db).await {
        error!("Failed to save repair event: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database error".to_string(),
        ));
    }

    // Intercept intake_save events
    if payload.event_type == "intake_save" {
        let state_clone = state.clone();
        let target_device_id = payload.target_device_id.clone();
        let data = payload.data.clone();
        tokio::spawn(async move {
            if let Err(e) =
                RepairService::process_intake(state_clone, target_device_id, &data).await
            {
                error!("Repair Service Error: {}", e);
            }
        });
    }

    // Intercept inventory count_submit
    if payload.event_type == "count_submit" && doc_type == "inventory_count" {
        let discrepancies = process_inventory_count(
            &state,
            &payload.source_device_id,
            &doc_id.to_string(),
            &payload.data,
        )
        .await;
        info!(
            "Inventory count: {} -> {} discrepancies",
            payload.source_device_id,
            discrepancies.len()
        );

        return Ok(Json(serde_json::json!({
            "success": true,
            "documentId": doc_id.to_string(),
            "message": "Inventory count processed",
            "discrepancies": discrepancies,
        })));
    }

    info!(
        "Repair event: {} -> {} ({}: {})",
        payload.source_device_id, payload.target_device_id, payload.event_type, payload.data
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "documentId": doc_id.to_string(),
        "message": "Repair event logged",
    })))
}

/// Compares physical count against server stock, updates StockQuant (PDA wins),
/// and records InventoryDiscrepancy for each mismatch.
/// Mirrors Go's `processInventoryCount` from `internal/handlers/repair.go`.
async fn process_inventory_count(
    state: &Arc<AppState>,
    device_id: &str,
    doc_id: &str,
    raw_data: &str,
) -> Vec<DiscrepancyResponse> {
    #[derive(Deserialize)]
    struct InvItem {
        barcode: String,
        quantity: i32,
        r#type: String,
    }
    #[derive(Deserialize)]
    struct InvPayload {
        location: String,
        items: Vec<InvItem>,
    }

    let payload: InvPayload = match serde_json::from_str(raw_data) {
        Ok(p) => p,
        Err(e) => {
            error!("Inventory: failed to parse payload: {}", e);
            return vec![];
        }
    };

    // Resolve location barcode -> StockLocation
    let mut location_id: i64 = 0;
    let mut location_name = payload.location.clone();
    let loc_found = if let Ok(Some(loc)) = location::Entity::find()
        .filter(location::Column::Barcode.eq(&payload.location))
        .one(&state.db)
        .await
    {
        location_id = loc.id;
        location_name = loc.complete_name;
        true
    } else {
        false
    };

    let mut discrepancies = Vec::new();

    for item in payload.items {
        // Resolve item barcode -> ProductProduct
        let (mut prod_found, prod) = match product::Entity::find()
            .filter(
                Condition::any()
                    .add(product::Column::Barcode.eq(&item.barcode))
                    .add(product::Column::DefaultCode.eq(&item.barcode)),
            )
            .one(&state.db)
            .await
        {
            Ok(Some(p)) => (true, p),
            _ => {
                // Source of Truth = PDA: create stub product
                warn!(
                    "Inventory: Unknown barcode {} — creating stub product (PDA is source of truth)",
                    item.barcode
                );
                let stub_id = -Utc::now().timestamp_micros();
                let stub = product::ActiveModel {
                    id: Set(stub_id),
                    name: Set(format!("[PDA] {}", item.barcode)),
                    barcode: Set(crate::models::odoo_types::OdooString(item.barcode.clone())),
                    default_code: Set(crate::models::odoo_types::OdooString(format!(
                        "PDA-{}",
                        item.barcode
                    ))),
                    active: Set(true),
                    r#type: Set("product".to_string()),
                    list_price: Set(0.0),
                    standard_price: Set(0.0),
                    weight: Set(0.0),
                    volume: Set(0.0),
                    write_date: Set(Utc::now()),
                    last_synced_at: Set(Utc::now()),
                };
                match stub.insert(&state.db).await {
                    Ok(p) => {
                        info!("Created stub product: {} (ID: {})", p.name, p.id);
                        (true, p)
                    }
                    Err(e) => {
                        error!(
                            "Failed to create stub product for {}: {}",
                            item.barcode, e
                        );
                        // Build a dummy model for discrepancy reporting
                        (
                            false,
                            product::Model {
                                id: stub_id,
                                name: format!("[PDA] {}", item.barcode),
                                barcode: crate::models::odoo_types::OdooString(
                                    item.barcode.clone(),
                                ),
                                default_code: crate::models::odoo_types::OdooString(format!(
                                    "PDA-{}",
                                    item.barcode
                                )),
                                active: true,
                                r#type: "product".to_string(),
                                list_price: 0.0,
                                standard_price: 0.0,
                                weight: 0.0,
                                volume: 0.0,
                                write_date: Utc::now(),
                                last_synced_at: Utc::now(),
                            },
                        )
                    }
                }
            }
        };

        let mut expected_qty = 0.0;
        let counted_qty = item.quantity as f64;

        if prod_found && loc_found {
            // Query current server stock at this location
            let quants = quant::Entity::find()
                .filter(quant::Column::ProductId.eq(prod.id))
                .filter(quant::Column::LocationId.eq(location_id))
                .all(&state.db)
                .await
                .unwrap_or_default();

            for q in &quants {
                expected_qty += q.quantity;
            }

            // Physical count wins — update StockQuant
            if quants.is_empty() {
                let new_quant = quant::ActiveModel {
                    id: Set(-Utc::now().timestamp_micros()),
                    product_id: Set(prod.id),
                    location_id: Set(location_id),
                    quantity: Set(counted_qty),
                    reserved_quantity: Set(0.0),
                    ..Default::default()
                };
                let _ = new_quant.insert(&state.db).await;
            } else {
                // Update first row to counted qty
                let mut q0: quant::ActiveModel = quants[0].clone().into();
                q0.quantity = Set(counted_qty);
                let _ = q0.update(&state.db).await;

                // Zero out remaining rows (different lots/packages)
                for q in quants.iter().skip(1) {
                    let mut qn: quant::ActiveModel = q.clone().into();
                    qn.quantity = Set(0.0);
                    let _ = qn.update(&state.db).await;
                }
            }
        }

        let delta = counted_qty - expected_qty;

        // Record discrepancy if mismatch
        if delta.abs() > 0.001 {
            let disc = inventory_discrepancy::ActiveModel {
                id: Set(Uuid::new_v4()),
                document_id: Set(doc_id.to_string()),
                product_id: Set(prod.id),
                product_barcode: Set(item.barcode.clone()),
                product_name: Set(prod.name.clone()),
                product_code: Set(prod.default_code.0.clone()),
                location_id: Set(location_id),
                location_barcode: Set(payload.location.clone()),
                location_name: Set(location_name.clone()),
                expected_qty: Set(expected_qty),
                counted_qty: Set(counted_qty),
                delta: Set(delta),
                item_type: Set(item.r#type.clone()),
                device_id: Set(device_id.to_string()),
                status: Set("pending".to_string()),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
                ..Default::default()
            };
            let _ = disc.insert(&state.db).await;

            discrepancies.push(DiscrepancyResponse {
                barcode: item.barcode.clone(),
                product_name: prod.name.clone(),
                product_code: prod.default_code.0.clone(),
                expected_qty,
                counted_qty,
                delta,
                location: location_name.clone(),
            });
        }

        let prod_label = if prod_found {
            prod.name.as_str()
        } else {
            item.barcode.as_str()
        };
        info!(
            "Inventory item: {} @ {} — counted:{:.0} expected:{:.0} delta:{:.0}",
            prod_label, location_name, counted_qty, expected_qty, delta
        );
    }

    discrepancies
}

/// GET /api/repair/events — lists repair log documents.
pub async fn list_repair_events(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<
        std::collections::HashMap<String, String>,
    >,
) -> Result<Json<Vec<document::Model>>, StatusCode> {
    let target_id = params.get("target_device_id");

    let mut docs = document::Entity::find()
        .filter(document::Column::Type.eq("repair_log"))
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(tid) = target_id {
        docs.retain(|d| {
            d.payload
                .get("target_device_id")
                .and_then(|v| v.as_str())
                == Some(tid.as_str())
        });
    }

    docs.truncate(100);
    Ok(Json(docs))
}
