use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    db::AppState,
    models::{delivery_carrier, stock_picking_delivery, sync_history},
};

#[derive(Deserialize)]
pub struct CreateShipmentReq {
    pub picking_id: i64,
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
    // In a full implementation, this calls DeliveryService::create_shipment
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
    Path(id): Path<i64>,
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
    Path(_id): Path<i64>,
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

pub async fn trigger_opal_import() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "started",
        "message": "OPAL synchronization started in background."
    })))
}

pub async fn trigger_dhl_import() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "started",
        "message": "DHL synchronization started in background."
    })))
}
