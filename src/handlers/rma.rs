use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use std::sync::Arc;

use crate::{db::AppState, models::order};

#[derive(Deserialize)]
pub struct ListQuery {
    pub r#type: Option<String>,
}

/// GET /rma — list all orders, optionally filtered by type.
/// Mirrors Go's `listOrders` from `internal/handlers/rma.go`.
pub async fn list_orders(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<order::Model>>, StatusCode> {
    let mut query = order::Entity::find();

    if let Some(ref order_type) = q.r#type {
        query = query.filter(order::Column::OrderType.eq(order_type));
    }

    let orders = query
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(orders))
}

/// GET /rma/:id — get a single order.
pub async fn get_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<order::Model>, StatusCode> {
    let o = order::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(o))
}

/// Request body for creating an order — all fields optional with serde defaults.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CreateOrderRequest {
    pub order_type: String,
    pub order_number: String,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: String,
    pub item_id: Option<i32>,
    pub product_sku: String,
    pub product_name: String,
    pub serial_number: String,
    pub purchase_date: Option<chrono::DateTime<Utc>>,
    pub issue_description: String,
    pub diagnosis_notes: String,
    pub assigned_to: Option<String>,
    pub status: String,
    pub priority: String,
    pub repair_notes: String,
    pub parts_used: Option<serde_json::Value>,
    pub labor_hours: f64,
    pub total_cost: f64,
    pub resolution: String,
    pub notes: String,
    pub metadata: Option<serde_json::Value>,
    pub rma_reason: String,
    pub is_refund_requested: bool,
}

/// POST /rma — create a new order.
/// Mirrors Go's `createOrder` from `internal/handlers/rma.go`.
pub async fn create_order(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<(StatusCode, Json<order::Model>), (StatusCode, String)> {
    let order_type = if payload.order_type.is_empty() {
        "rma".to_string()
    } else {
        payload.order_type
    };

    let order_number = if payload.order_number.is_empty() {
        let prefix = match order_type.as_str() {
            "rma" => "RMA",
            "repair" => "REP",
            _ => "ORD",
        };
        format!(
            "{}{}-{}",
            prefix,
            Utc::now().format("%Y%m%d"),
            rand::random::<u16>()
        )
    } else {
        payload.order_number
    };

    let am = order::ActiveModel {
        order_number: Set(order_number),
        order_type: Set(order_type),
        customer_name: Set(payload.customer_name),
        customer_email: Set(payload.customer_email),
        customer_phone: Set(payload.customer_phone),
        item_id: Set(payload.item_id),
        product_sku: Set(payload.product_sku),
        product_name: Set(payload.product_name),
        serial_number: Set(payload.serial_number),
        purchase_date: Set(payload.purchase_date),
        issue_description: Set(payload.issue_description),
        diagnosis_notes: Set(payload.diagnosis_notes),
        assigned_to: Set(payload.assigned_to),
        status: Set(if payload.status.is_empty() {
            "pending".to_string()
        } else {
            payload.status
        }),
        priority: Set(if payload.priority.is_empty() {
            "normal".to_string()
        } else {
            payload.priority
        }),
        repair_notes: Set(payload.repair_notes),
        parts_used: Set(payload.parts_used.unwrap_or(serde_json::json!([]))),
        labor_hours: Set(payload.labor_hours),
        total_cost: Set(payload.total_cost),
        resolution: Set(payload.resolution),
        notes: Set(payload.notes),
        metadata: Set(payload.metadata.unwrap_or(serde_json::json!({}))),
        rma_reason: Set(payload.rma_reason),
        is_refund_requested: Set(payload.is_refund_requested),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    let inserted = am
        .insert(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(inserted)))
}

/// PUT /rma/:id — update an existing order.
/// Mirrors Go's `updateOrder` from `internal/handlers/rma.go`.
pub async fn update_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<Json<order::Model>, (StatusCode, String)> {
    let existing = order::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Order not found".to_string()))?;

    let mut am: order::ActiveModel = existing.into();
    am.customer_name = Set(payload.customer_name);
    am.customer_email = Set(payload.customer_email);
    am.customer_phone = Set(payload.customer_phone);
    am.product_sku = Set(payload.product_sku);
    am.product_name = Set(payload.product_name);
    am.serial_number = Set(payload.serial_number);
    am.issue_description = Set(payload.issue_description);
    am.diagnosis_notes = Set(payload.diagnosis_notes);
    am.assigned_to = Set(payload.assigned_to);
    am.status = Set(payload.status);
    am.priority = Set(payload.priority);
    am.repair_notes = Set(payload.repair_notes);
    if let Some(parts) = payload.parts_used {
        am.parts_used = Set(parts);
    }
    am.labor_hours = Set(payload.labor_hours);
    am.total_cost = Set(payload.total_cost);
    am.resolution = Set(payload.resolution);
    am.notes = Set(payload.notes);
    am.rma_reason = Set(payload.rma_reason);
    am.is_refund_requested = Set(payload.is_refund_requested);
    am.updated_at = Set(Utc::now());

    let updated = am
        .update(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(updated))
}

/// DELETE /rma/:id — soft delete an order.
pub async fn delete_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    order::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"message": "Order deleted successfully"})))
}
