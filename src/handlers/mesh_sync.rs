use axum::{extract::State, http::StatusCode, Json};
use sea_orm::{ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::AppState;
use crate::models::{location, product, stock_picking_delivery};
use crate::sync::merkle_tree::{MerkleNode, MerkleRequest, MerkleTreeService};

// --- MERKLE ---

/// POST /mesh/merkle — Get Merkle tree state for comparison
pub async fn merkle_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MerkleRequest>,
) -> Result<Json<MerkleNode>, (StatusCode, String)> {
    let svc = MerkleTreeService::new(&state.db);
    let node = svc
        .get_state(&payload)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(node))
}

// --- PULL ---

#[derive(Deserialize)]
pub struct PullRequest {
    pub entity_type: String,
    pub ids: Vec<String>,
}

#[derive(Serialize)]
pub struct PullResponse {
    pub entity_type: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub products: Vec<product::Model>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<location::Model>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub shipments: Vec<stock_picking_delivery::Model>,
}

/// POST /mesh/pull — Fetch specific entities by ID for sync
pub async fn pull_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PullRequest>,
) -> Result<Json<PullResponse>, (StatusCode, String)> {
    let mut resp = PullResponse {
        entity_type: payload.entity_type.clone(),
        products: vec![],
        locations: vec![],
        shipments: vec![],
    };

    let parsed_ids: Vec<i64> = payload
        .ids
        .iter()
        .filter_map(|s| s.parse().ok())
        .collect();

    match payload.entity_type.as_str() {
        "product" => {
            resp.products = product::Entity::find()
                .filter(product::Column::Id.is_in(parsed_ids))
                .all(&state.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
        "location" => {
            resp.locations = location::Entity::find()
                .filter(location::Column::Id.is_in(parsed_ids))
                .all(&state.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
        "shipment" => {
            resp.shipments = stock_picking_delivery::Entity::find()
                .filter(stock_picking_delivery::Column::Id.is_in(parsed_ids))
                .all(&state.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
        other => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Unknown entity_type: {}", other),
            ));
        }
    }

    Ok(Json(resp))
}

// --- PUSH ---

#[derive(Deserialize)]
pub struct PushPayload {
    #[serde(default)]
    pub products: Vec<product::Model>,
    #[serde(default)]
    pub locations: Vec<location::Model>,
    #[serde(default)]
    pub shipments: Vec<stock_picking_delivery::Model>,
}

/// POST /mesh/push — Apply incoming entities (upsert)
pub async fn push_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PushPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let db = &state.db;
    let mut applied = 0u32;

    for p in payload.products {
        let am = p.into_active_model();
        let _ = product::Entity::insert(am)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(product::Column::Id)
                    .update_columns([
                        product::Column::DefaultCode,
                        product::Column::Barcode,
                        product::Column::Name,
                        product::Column::Active,
                        product::Column::Type,
                        product::Column::ListPrice,
                        product::Column::StandardPrice,
                        product::Column::Weight,
                        product::Column::Volume,
                        product::Column::WriteDate,
                        product::Column::LastSyncedAt,
                    ])
                    .to_owned(),
            )
            .exec(db)
            .await;
        applied += 1;
    }

    for l in payload.locations {
        let am = l.into_active_model();
        let _ = location::Entity::insert(am)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(location::Column::Id)
                    .update_columns([
                        location::Column::Name,
                        location::Column::CompleteName,
                        location::Column::Barcode,
                        location::Column::Usage,
                        location::Column::LocationId,
                        location::Column::Active,
                        location::Column::LastSyncedAt,
                        location::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(db)
            .await;
        applied += 1;
    }

    for s in payload.shipments {
        let am = s.into_active_model();
        let _ = stock_picking_delivery::Entity::insert(am)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(stock_picking_delivery::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await;
        applied += 1;
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "applied": applied
    })))
}
