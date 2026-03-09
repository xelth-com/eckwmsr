use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    db::AppState,
    models::{delivery_carrier, stock_picking_delivery, sync_history},
};

#[derive(Deserialize)]
pub struct CreateShipmentReq {
    pub picking_id: Uuid,
    pub provider_code: String,
}

#[derive(Deserialize)]
pub struct ShipmentQuery {
    pub _state: Option<String>,
    pub limit: Option<u64>,
}

pub async fn get_delivery_config(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let config = serde_json::json!({
        "opal": !std::env::var("OPAL_USERNAME").unwrap_or_default().is_empty(),
        "dhl": !std::env::var("DHL_USERNAME").unwrap_or_default().is_empty(),
    });
    Ok(Json(config))
}

pub async fn create_shipment(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<CreateShipmentReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    Ok(Json(serde_json::json!({
        "message": "Shipment created and queued for processing",
        "picking_id": payload.picking_id,
        "provider": payload.provider_code
    })))
}

pub async fn list_shipments(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ShipmentQuery>,
) -> Result<Json<Vec<stock_picking_delivery::Model>>, StatusCode> {
    let limit = q.limit.unwrap_or(50);

    let shipments = stock_picking_delivery::Entity::find()
        .order_by_desc(stock_picking_delivery::Column::CreatedAt)
        .limit(limit)
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(shipments))
}

pub async fn get_shipment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<stock_picking_delivery::Model>, (StatusCode, String)> {
    let shipment = stock_picking_delivery::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Shipment not found".to_string()))?;

    Ok(Json(shipment))
}

pub async fn cancel_shipment(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({"status": "cancelled"})))
}

pub async fn list_carriers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<delivery_carrier::Model>>, StatusCode> {
    let carriers = delivery_carrier::Entity::find()
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(carriers))
}

pub async fn get_sync_history(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<sync_history::Model>>, StatusCode> {
    let history = sync_history::Entity::find()
        .order_by_desc(sync_history::Column::StartedAt)
        .limit(100u64)
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(history))
}

// ─── OPAL Import ─────────────────────────────────────────────────────────────

pub async fn trigger_opal_import(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let username = std::env::var("OPAL_USERNAME").unwrap_or_default();
    let password = std::env::var("OPAL_PASSWORD").unwrap_or_default();
    let url = std::env::var("OPAL_URL")
        .unwrap_or_else(|_| "https://opal-kurier.de".to_string());

    if username.is_empty() {
        return Ok(Json(serde_json::json!({
            "status": "error",
            "message": "OPAL_USERNAME not configured"
        })));
    }

    let state = state.clone();
    tokio::spawn(async move {
        perform_opal_import(state, username, password, url, 50).await;
    });

    Ok(Json(serde_json::json!({
        "status": "started",
        "message": "OPAL synchronization started in background."
    })))
}

async fn perform_opal_import(
    state: Arc<AppState>,
    username: String,
    password: String,
    _url: String,
    limit: u32,
) {
    let sync_id = Uuid::new_v4().to_string();
    let started_at = Utc::now();
    let instance_id = state.config.instance_id.clone();

    // Insert sync_history record with status "running"
    let history = sync_history::ActiveModel {
        id: Set(sync_id.clone()),
        instance_id: Set(instance_id.clone()),
        provider: Set("opal".to_string()),
        status: Set("running".to_string()),
        started_at: Set(started_at),
        completed_at: Set(None),
        duration: Set(0),
        created: Set(0),
        updated: Set(0),
        skipped: Set(0),
        errors: Set(0),
        error_detail: Set(String::new()),
        debug_info: Set(None),
        created_at: Set(started_at),
        updated_at: Set(started_at),
        deleted_at: Set(None),
    };
    if let Err(e) = history.insert(&state.db).await {
        error!("[OPAL] Failed to create sync_history: {}", e);
        return;
    }
    info!("[OPAL] Import started, sync_id={}", sync_id);

    // Ensure OPAL carrier exists
    let carrier_id = match ensure_carrier(&state, "opal", "OPAL Kurier").await {
        Some(id) => id,
        None => {
            finish_sync(&state, &sync_id, "error", 0, 0, 0, 1, "Failed to create carrier", started_at).await;
            return;
        }
    };

    // Call Node.js scraper
    let scraper_url = "http://127.0.0.1:3211/api/opal/fetch";
    let body = serde_json::json!({
        "username": username,
        "password": password,
        "limit": limit
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .unwrap_or_default();

    let resp = match client.post(scraper_url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            error!("[OPAL] Scraper call failed: {}", e);
            finish_sync(&state, &sync_id, "error", 0, 0, 0, 1,
                &format!("Scraper unreachable: {}", e), started_at).await;
            return;
        }
    };

    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(e) => {
            error!("[OPAL] Failed to parse scraper response: {}", e);
            finish_sync(&state, &sync_id, "error", 0, 0, 0, 1,
                &format!("Parse error: {}", e), started_at).await;
            return;
        }
    };

    if json.get("success").and_then(|v| v.as_bool()) != Some(true) {
        let msg = json.get("error").and_then(|v| v.as_str()).unwrap_or("unknown");
        error!("[OPAL] Scraper returned error: {}", msg);
        finish_sync(&state, &sync_id, "error", 0, 0, 0, 1, msg, started_at).await;
        return;
    }

    let orders = match json.get("orders").and_then(|v| v.as_array()) {
        Some(o) => o.clone(),
        None => {
            warn!("[OPAL] No orders array in response");
            finish_sync(&state, &sync_id, "success", 0, 0, 0, 0, "", started_at).await;
            return;
        }
    };

    info!("[OPAL] Processing {} orders", orders.len());

    let mut created = 0i64;
    let mut updated = 0i64;
    let mut skipped = 0i64;
    let mut errors = 0i64;

    for order in &orders {
        let tracking = order.get("tracking_number")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        if tracking.is_empty() {
            skipped += 1;
            continue;
        }

        let status = map_opal_status(
            order.get("status").and_then(|v| v.as_str()).unwrap_or(""),
        );

        let delivered_at = if status == "delivered" {
            parse_opal_datetime(
                order.get("status_date").and_then(|v| v.as_str()).unwrap_or(""),
                order.get("status_time").and_then(|v| v.as_str()).unwrap_or(""),
            )
        } else {
            None
        };

        let shipped_at = parse_opal_date(
            order.get("pickup_date").and_then(|v| v.as_str()).unwrap_or(""),
        );

        let last_activity_at = parse_opal_datetime(
            order.get("status_date").and_then(|v| v.as_str()).unwrap_or(""),
            order.get("status_time").and_then(|v| v.as_str()).unwrap_or(""),
        ).or(Some(Utc::now()));

        let raw_response = order.to_string();

        // Check if already exists by tracking_number
        let existing = stock_picking_delivery::Entity::find()
            .filter(stock_picking_delivery::Column::TrackingNumber.eq(&tracking))
            .one(&state.db)
            .await
            .unwrap_or(None);

        if let Some(existing) = existing {
            // Update status and dates
            use sea_orm::IntoActiveModel;
            let mut active = existing.into_active_model();
            active.status = Set(status);
            active.delivered_at = Set(delivered_at);
            active.last_activity_at = Set(last_activity_at);
            active.raw_response = Set(raw_response);
            active.updated_at = Set(Utc::now());
            match active.update(&state.db).await {
                Ok(_) => updated += 1,
                Err(e) => { error!("[OPAL] Update failed for {}: {}", tracking, e); errors += 1; }
            }
        } else {
            // Insert new
            let new_row = stock_picking_delivery::ActiveModel {
                picking_id: Set(None),
                carrier_id: Set(Some(carrier_id)),
                tracking_number: Set(tracking.clone()),
                carrier_price: Set(0.0),
                currency: Set("EUR".to_string()),
                status: Set(status),
                error_message: Set(String::new()),
                label_url: Set(String::new()),
                label_data: Set(None),
                raw_response: Set(raw_response),
                shipped_at: Set(shipped_at),
                delivered_at: Set(delivered_at),
                last_activity_at: Set(last_activity_at),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
                ..Default::default()
            };
            match new_row.insert(&state.db).await {
                Ok(_) => { created += 1; info!("[OPAL] Saved: {}", tracking); }
                Err(e) => { error!("[OPAL] Insert failed for {}: {}", tracking, e); errors += 1; }
            }
        }
    }

    let final_status = if errors > 0 && created + updated == 0 { "error" } else { "success" };
    info!("[OPAL] Import done: created={} updated={} skipped={} errors={}", created, updated, skipped, errors);
    finish_sync(&state, &sync_id, final_status, created, updated, skipped, errors, "", started_at).await;
}

// ─── DHL Import ──────────────────────────────────────────────────────────────

pub async fn trigger_dhl_import(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let username = std::env::var("DHL_USERNAME").unwrap_or_default();
    let password = std::env::var("DHL_PASSWORD").unwrap_or_default();
    let url = std::env::var("DHL_URL")
        .unwrap_or_else(|_| "https://geschaeftskunden.dhl.de".to_string());

    if username.is_empty() {
        return Ok(Json(serde_json::json!({
            "status": "error",
            "message": "DHL_USERNAME not configured"
        })));
    }

    let state = state.clone();
    tokio::spawn(async move {
        perform_dhl_import(state, username, password, url).await;
    });

    Ok(Json(serde_json::json!({
        "status": "started",
        "message": "DHL synchronization started in background."
    })))
}

async fn perform_dhl_import(
    state: Arc<AppState>,
    username: String,
    password: String,
    url: String,
) {
    let sync_id = Uuid::new_v4().to_string();
    let started_at = Utc::now();
    let instance_id = state.config.instance_id.clone();

    let history = sync_history::ActiveModel {
        id: Set(sync_id.clone()),
        instance_id: Set(instance_id),
        provider: Set("dhl".to_string()),
        status: Set("running".to_string()),
        started_at: Set(started_at),
        completed_at: Set(None),
        duration: Set(0),
        created: Set(0),
        updated: Set(0),
        skipped: Set(0),
        errors: Set(0),
        error_detail: Set(String::new()),
        debug_info: Set(None),
        created_at: Set(started_at),
        updated_at: Set(started_at),
        deleted_at: Set(None),
    };
    if let Err(e) = history.insert(&state.db).await {
        error!("[DHL] Failed to create sync_history: {}", e);
        return;
    }
    info!("[DHL] Import started, sync_id={}", sync_id);

    let carrier_id = match ensure_carrier(&state, "dhl", "DHL").await {
        Some(id) => id,
        None => {
            finish_sync(&state, &sync_id, "error", 0, 0, 0, 1, "Failed to create carrier", started_at).await;
            return;
        }
    };

    let scraper_url = "http://127.0.0.1:3211/api/dhl/fetch";
    let body = serde_json::json!({
        "username": username,
        "password": password,
        "url": url
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .unwrap_or_default();

    let resp = match client.post(scraper_url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            error!("[DHL] Scraper call failed: {}", e);
            finish_sync(&state, &sync_id, "error", 0, 0, 0, 1,
                &format!("Scraper unreachable: {}", e), started_at).await;
            return;
        }
    };

    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(e) => {
            error!("[DHL] Failed to parse scraper response: {}", e);
            finish_sync(&state, &sync_id, "error", 0, 0, 0, 1,
                &format!("Parse error: {}", e), started_at).await;
            return;
        }
    };

    if json.get("success").and_then(|v| v.as_bool()) != Some(true) {
        let msg = json.get("error").and_then(|v| v.as_str()).unwrap_or("unknown");
        error!("[DHL] Scraper returned error: {}", msg);
        finish_sync(&state, &sync_id, "error", 0, 0, 0, 1, msg, started_at).await;
        return;
    }

    let shipments = match json.get("shipments").and_then(|v| v.as_array()) {
        Some(s) => s.clone(),
        None => {
            finish_sync(&state, &sync_id, "success", 0, 0, 0, 0, "", started_at).await;
            return;
        }
    };

    info!("[DHL] Processing {} shipments", shipments.len());

    let mut created = 0i64;
    let mut updated = 0i64;
    let mut skipped = 0i64;
    let mut errors = 0i64;

    for shipment in &shipments {
        let tracking = shipment.get("tracking_number")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        if tracking.is_empty() {
            skipped += 1;
            continue;
        }

        let status = map_dhl_status(
            shipment.get("status").and_then(|v| v.as_str()).unwrap_or(""),
        );
        let raw_response = shipment.to_string();

        let existing = stock_picking_delivery::Entity::find()
            .filter(stock_picking_delivery::Column::TrackingNumber.eq(&tracking))
            .one(&state.db)
            .await
            .unwrap_or(None);

        if let Some(existing) = existing {
            use sea_orm::IntoActiveModel;
            let mut active = existing.into_active_model();
            active.status = Set(status);
            active.raw_response = Set(raw_response);
            active.updated_at = Set(Utc::now());
            match active.update(&state.db).await {
                Ok(_) => updated += 1,
                Err(e) => { error!("[DHL] Update failed for {}: {}", tracking, e); errors += 1; }
            }
        } else {
            let new_row = stock_picking_delivery::ActiveModel {
                picking_id: Set(None),
                carrier_id: Set(Some(carrier_id)),
                tracking_number: Set(tracking.clone()),
                carrier_price: Set(0.0),
                currency: Set("EUR".to_string()),
                status: Set(status),
                error_message: Set(String::new()),
                label_url: Set(String::new()),
                label_data: Set(None),
                raw_response: Set(raw_response),
                shipped_at: Set(None),
                delivered_at: Set(None),
                last_activity_at: Set(Some(Utc::now())),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
                ..Default::default()
            };
            match new_row.insert(&state.db).await {
                Ok(_) => { created += 1; info!("[DHL] Saved: {}", tracking); }
                Err(e) => { error!("[DHL] Insert failed for {}: {}", tracking, e); errors += 1; }
            }
        }
    }

    let final_status = if errors > 0 && created + updated == 0 { "error" } else { "success" };
    info!("[DHL] Import done: created={} updated={} skipped={} errors={}", created, updated, skipped, errors);
    finish_sync(&state, &sync_id, final_status, created, updated, skipped, errors, "", started_at).await;
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Get or create a delivery carrier by provider_code
async fn ensure_carrier(state: &Arc<AppState>, code: &str, name: &str) -> Option<Uuid> {
    if let Ok(Some(c)) = delivery_carrier::Entity::find()
        .filter(delivery_carrier::Column::ProviderCode.eq(code))
        .one(&state.db)
        .await
    {
        return Some(c.id);
    }

    let new_carrier = delivery_carrier::ActiveModel {
        name: Set(name.to_string()),
        provider_code: Set(code.to_string()),
        active: Set(true),
        config_json: Set(String::new()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    match new_carrier.insert(&state.db).await {
        Ok(c) => Some(c.id),
        Err(e) => { error!("Failed to insert carrier '{}': {}", code, e); None }
    }
}

/// Update sync_history record to final state
async fn finish_sync(
    state: &Arc<AppState>,
    sync_id: &str,
    status: &str,
    created: i64,
    updated: i64,
    skipped: i64,
    errors: i64,
    error_detail: &str,
    started_at: chrono::DateTime<Utc>,
) {
    let now = Utc::now();
    let duration = (now - started_at).num_seconds();

    if let Ok(Some(record)) = sync_history::Entity::find_by_id(sync_id.to_string())
        .one(&state.db)
        .await
    {
        use sea_orm::IntoActiveModel;
        let mut active = record.into_active_model();
        active.status = Set(status.to_string());
        active.completed_at = Set(Some(now));
        active.duration = Set(duration);
        active.created = Set(created);
        active.updated = Set(updated);
        active.skipped = Set(skipped);
        active.errors = Set(errors);
        active.error_detail = Set(error_detail.to_string());
        active.updated_at = Set(now);
        let _ = active.update(&state.db).await;
    }
}

/// Map OPAL status string to our internal status
fn map_opal_status(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "zugestellt" | "ausgeliefert" | "geliefert" => "delivered",
        "aktiv" => "in_transit",
        "abgeholt" => "picked_up",
        "storniert" => "cancelled",
        "fehlanfahrt" => "failed",
        _ => "unknown",
    }
    .to_string()
}

/// Map DHL status string to our internal status
fn map_dhl_status(s: &str) -> String {
    let lower = s.to_lowercase();
    if lower.contains("zugestellt") || lower.contains("delivered") {
        "delivered".to_string()
    } else if lower.contains("unterwegs") || lower.contains("transit") || lower.contains("sortiert") {
        "in_transit".to_string()
    } else if lower.contains("storniert") || lower.contains("cancelled") {
        "cancelled".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Parse OPAL date "dd.mm.yy" or "dd.mm.yyyy" to DateTimeUtc
fn parse_opal_date(date_str: &str) -> Option<chrono::DateTime<Utc>> {
    parse_opal_datetime(date_str, "00:00")
}

/// Parse OPAL date "dd.mm.yy" + time "HH:MM" to DateTimeUtc
fn parse_opal_datetime(date_str: &str, time_str: &str) -> Option<chrono::DateTime<Utc>> {
    if date_str.is_empty() {
        return None;
    }
    let parts: Vec<&str> = date_str.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let day: u32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let year_raw: i32 = parts[2].parse().ok()?;
    let year = if year_raw < 100 { year_raw + 2000 } else { year_raw };

    let time_parts: Vec<&str> = time_str.split(':').collect();
    let hour: u32 = time_parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let min: u32 = time_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

    use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
    let naive = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(year, month, day)?,
        NaiveTime::from_hms_opt(hour, min, 0)?,
    );
    Some(chrono::Utc.from_utc_datetime(&naive))
}
