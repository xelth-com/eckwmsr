use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::{db::AppState, models::inventory_discrepancy};

#[derive(Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

/// GET /api/inventory/discrepancies — list discrepancies with optional status filter.
/// Mirrors Go's `listDiscrepancies` from `internal/handlers/inventory.go`.
pub async fn list_discrepancies(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut query = inventory_discrepancy::Entity::find()
        .order_by_desc(inventory_discrepancy::Column::CreatedAt);

    if let Some(ref status) = q.status {
        query = query.filter(inventory_discrepancy::Column::Status.eq(status));
    }

    let total = query
        .clone()
        .count(&state.db)
        .await
        .unwrap_or(0);

    let limit = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);

    let items = query
        .limit(limit)
        .offset(offset)
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "items": items,
        "total": total,
        "limit": limit,
        "offset": offset,
    })))
}

/// GET /api/inventory/discrepancies/:id — get a single discrepancy.
pub async fn get_discrepancy(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<inventory_discrepancy::Model>, StatusCode> {
    let disc = inventory_discrepancy::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(disc))
}

#[derive(Deserialize)]
pub struct ReviewRequest {
    pub status: String,
    pub notes: String,
}

/// PUT /api/inventory/discrepancies/:id/review — update discrepancy status.
/// Mirrors Go's `reviewDiscrepancy` from `internal/handlers/inventory.go`.
pub async fn review_discrepancy(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReviewRequest>,
) -> Result<Json<inventory_discrepancy::Model>, (StatusCode, String)> {
    if payload.status != "reviewed" && payload.status != "resolved" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Status must be 'reviewed' or 'resolved'".to_string(),
        ));
    }

    let disc = inventory_discrepancy::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Not found".to_string()))?;

    let mut am: inventory_discrepancy::ActiveModel = disc.into();
    am.status = Set(payload.status);
    am.notes = Set(Some(payload.notes));
    am.reviewed_at = Set(Some(Utc::now()));
    am.updated_at = Set(Utc::now());

    let updated = am
        .update(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(updated))
}

/// GET /api/inventory/discrepancies/stats — aggregate counts by status.
/// Mirrors Go's `getDiscrepancyStats` from `internal/handlers/inventory.go`.
pub async fn get_discrepancy_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HashMap<String, i64>>, StatusCode> {
    let mut stats = HashMap::from([
        ("pending".to_string(), 0i64),
        ("reviewed".to_string(), 0i64),
        ("resolved".to_string(), 0i64),
    ]);

    #[derive(sea_orm::FromQueryResult)]
    struct StatusCount {
        status: String,
        count: i64,
    }

    let counts: Vec<StatusCount> = inventory_discrepancy::Entity::find()
        .select_only()
        .column(inventory_discrepancy::Column::Status)
        .column_as(inventory_discrepancy::Column::Id.count(), "count")
        .group_by(inventory_discrepancy::Column::Status)
        .into_model::<StatusCount>()
        .all(&state.db)
        .await
        .unwrap_or_default();

    for c in counts {
        stats.insert(c.status, c.count);
    }

    Ok(Json(stats))
}
