use axum::{extract::State, http::StatusCode, Json};
use sea_orm::*;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;

use crate::{
    db::AppState,
    models::{location, product, quant},
};

#[derive(Serialize)]
pub struct ProductWithQty {
    #[serde(flatten)]
    pub product: product::Model,
    pub qty_available: f64,
}

/// GET /api/warehouse — lists all stock locations ordered by complete_name.
/// Mirrors Go's `listWarehouses` from `internal/handlers/warehouse.go`.
pub async fn list_warehouses(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<location::Model>>, StatusCode> {
    let locs = location::Entity::find()
        .order_by_asc(location::Column::CompleteName)
        .all(&state.db)
        .await
        .map_err(|e| { error!("DB query error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(locs))
}

/// GET /api/items — lists products enriched with qty_available from stock_quant.
/// Mirrors Go's `listItems` from `internal/handlers/warehouse.go`.
pub async fn list_items(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ProductWithQty>>, StatusCode> {
    let products = product::Entity::find()
        .all(&state.db)
        .await
        .map_err(|e| { error!("DB query error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    // Aggregate quants in memory (matches Go's SELECT product_id, SUM(quantity) GROUP BY)
    let quants = quant::Entity::find()
        .all(&state.db)
        .await
        .map_err(|e| { error!("DB query error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    let mut qty_map: HashMap<i64, f64> = HashMap::new();
    for q in quants {
        *qty_map.entry(q.product_id).or_insert(0.0) += q.quantity;
    }

    let result = products
        .into_iter()
        .map(|p| {
            let qty = qty_map.get(&p.id).copied().unwrap_or(0.0);
            ProductWithQty {
                product: p,
                qty_available: qty,
            }
        })
        .collect();

    Ok(Json(result))
}
