use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::{
    db::AppState,
    models::{location, move_line, partner, picking, product, rack},
    utils::route::{calculate_route, PickStop, RouteResult},
};

// --- DTOs ---

#[derive(Serialize)]
pub struct EnrichedPicking {
    pub id: i64,
    pub name: String,
    pub state: String,
    pub location_id: i64,
    pub location_dest_id: i64,
    pub scheduled_date: chrono::DateTime<Utc>,
    pub origin: String,
    pub priority: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partner_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_done: Option<chrono::DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partner_name: Option<String>,
    pub line_count: i64,
    pub picked_count: i64,
}

#[derive(Serialize)]
pub struct EnrichedPickLine {
    pub id: i64,
    pub picking_id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub product_barcode: String,
    pub product_code: String,
    pub qty_done: f64,
    pub location_id: i64,
    pub location_name: String,
    pub location_barcode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rack_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rack_name: Option<String>,
    pub rack_x: i32,
    pub rack_y: i32,
    pub rack_width: i32,
    pub rack_height: i32,
    pub state: String,
    pub sequence: i32,
}

#[derive(Deserialize)]
pub struct ConfirmRequest {
    pub qty_done: f64,
    pub scanned_product_barcode: String,
    pub scanned_location_barcode: String,
}

#[derive(Serialize)]
pub struct RouteResponse {
    pub lines: Vec<EnrichedPickLine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<RouteResult>,
}

// --- Query params ---

#[derive(Deserialize)]
pub struct PickingQueryParams {
    pub state: Option<String>,
}

// --- Handlers ---

/// GET /api/odoo/pickings?state=assigned — lists pickings, optionally filtered by state.
/// Mirrors Go's `listOdooPickings` which returns raw StockPicking rows.
pub async fn list_odoo_pickings(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<PickingQueryParams>,
) -> Result<Json<Vec<picking::Model>>, StatusCode> {
    let mut query = picking::Entity::find();
    if let Some(ref s) = params.state {
        query = query.filter(picking::Column::State.eq(s.as_str()));
    }
    let pickings = query
        .order_by_desc(picking::Column::ScheduledDate)
        .all(&state.db)
        .await
        .map_err(|e| { tracing::error!("DB query error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok(Json(pickings))
}

/// GET /api/pickings/active — lists all assigned pickings with partner name and line counts.
/// Mirrors Go's `listActivePickings` from `internal/handlers/picking.go`.
pub async fn list_active_pickings(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<EnrichedPicking>>, StatusCode> {
    let pickings = picking::Entity::find()
        .filter(picking::Column::State.eq("assigned"))
        .order_by_desc(picking::Column::Priority)
        .order_by_asc(picking::Column::ScheduledDate)
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut result = Vec::with_capacity(pickings.len());

    for p in pickings {
        let partner_name = match p.partner_id {
            Some(pid) => partner::Entity::find_by_id(pid)
                .one(&state.db)
                .await
                .ok()
                .flatten()
                .map(|pr| pr.name),
            None => None,
        };

        let line_count = move_line::Entity::find()
            .filter(move_line::Column::PickingId.eq(p.id))
            .count(&state.db)
            .await
            .unwrap_or(0) as i64;

        let picked_count = move_line::Entity::find()
            .filter(
                Condition::all()
                    .add(move_line::Column::PickingId.eq(p.id))
                    .add(move_line::Column::State.eq("done")),
            )
            .count(&state.db)
            .await
            .unwrap_or(0) as i64;

        result.push(EnrichedPicking {
            id: p.id,
            name: p.name,
            state: p.state,
            location_id: p.location_id,
            location_dest_id: p.location_dest_id,
            scheduled_date: p.scheduled_date,
            origin: p.origin.0,
            priority: p.priority,
            partner_id: p.partner_id,
            date_done: p.date_done,
            partner_name,
            line_count,
            picked_count,
        });
    }

    Ok(Json(result))
}

/// GET /api/pickings/:id/lines — returns enriched pick lines for a picking.
/// Mirrors Go's `getPickingLines` from `internal/handlers/picking.go`.
pub async fn get_picking_lines(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<EnrichedPickLine>>, (StatusCode, String)> {
    let lines = enrich_pick_lines(&state.db, id).await?;
    Ok(Json(lines))
}

/// POST /api/pickings/:id/lines/:line_id/confirm — validates barcodes and updates a pick line.
/// Mirrors Go's `confirmPickLine` from `internal/handlers/picking.go`.
pub async fn confirm_pick_line(
    State(state): State<Arc<AppState>>,
    Path((picking_id, line_id)): Path<(i64, i64)>,
    Json(payload): Json<ConfirmRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let line = move_line::Entity::find()
        .filter(
            Condition::all()
                .add(move_line::Column::Id.eq(line_id))
                .add(move_line::Column::PickingId.eq(picking_id)),
        )
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Pick line not found".to_string()))?;

    // Validate product barcode
    if !payload.scanned_product_barcode.is_empty() {
        let prod = product::Entity::find_by_id(line.product_id)
            .one(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .ok_or_else(|| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Product not found".to_string(),
                )
            })?;

        if prod.barcode.0 != payload.scanned_product_barcode {
            return Err((
                StatusCode::CONFLICT,
                format!("Product barcode mismatch: expected {}", prod.barcode.0),
            ));
        }
    }

    // Validate location barcode
    if !payload.scanned_location_barcode.is_empty() {
        let loc = location::Entity::find_by_id(line.location_id)
            .one(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .ok_or_else(|| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Location not found".to_string(),
                )
            })?;

        if loc.barcode.0 != payload.scanned_location_barcode {
            return Err((
                StatusCode::CONFLICT,
                format!("Location barcode mismatch: expected {}", loc.barcode.0),
            ));
        }
    }

    // Compute new state before converting to ActiveModel
    // qty_demand not in DB table — treat as qty_done for state logic
    let new_state = if payload.qty_done > 0.0 {
        "done".to_string()
    } else {
        line.state.clone()
    };

    let mut active_line: move_line::ActiveModel = line.into();
    active_line.qty_done = Set(payload.qty_done);
    active_line.state = Set(new_state);
    let updated = active_line
        .update(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    info!(
        "Picking: Confirmed line {} (picking {}) qty_done={:.1}",
        line_id, picking_id, payload.qty_done
    );

    Ok(Json(serde_json::json!({
        "id": updated.id,
        "qty_done": updated.qty_done,
        "state": updated.state,
    })))
}

/// POST /api/pickings/:id/validate — marks a picking as done if all lines are complete.
/// Mirrors Go's `validatePicking` from `internal/handlers/picking.go`.
pub async fn validate_picking(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let pk = picking::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Picking not found".to_string()))?;

    if pk.state == "done" {
        return Err((StatusCode::CONFLICT, "Picking already completed".to_string()));
    }

    // Check for unfinished lines: state != 'done'
    let unfinished = move_line::Entity::find()
        .filter(
            Condition::all()
                .add(move_line::Column::PickingId.eq(id))
                .add(move_line::Column::State.ne("done")),
        )
        .count(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if unfinished > 0 {
        return Err((
            StatusCode::PRECONDITION_FAILED,
            format!("Cannot validate: {} lines not yet completed", unfinished),
        ));
    }

    let mut active_pk: picking::ActiveModel = pk.into();
    active_pk.state = Set("done".to_string());
    active_pk.date_done = Set(Some(Utc::now().into()));

    let updated = active_pk
        .update(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    info!("Picking: Validated {} (ID {})", updated.name, updated.id);

    Ok(Json(serde_json::json!({
        "id": updated.id,
        "name": updated.name,
        "state": updated.state,
        "date_done": updated.date_done,
    })))
}

/// GET /api/pickings/:id/route — returns lines ordered by TSP route optimization.
/// Mirrors Go's `getPickingRoute` from `internal/handlers/picking.go`.
pub async fn get_picking_route(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<RouteResponse>, (StatusCode, String)> {
    let mut lines = enrich_pick_lines(&state.db, id).await?;

    if lines.is_empty() {
        return Ok(Json(RouteResponse {
            lines,
            route: None,
        }));
    }

    // Group lines by rack — build pick stops
    let mut rack_stops: HashMap<i64, PickStop> = HashMap::new();
    let mut lines_by_rack: HashMap<i64, Vec<usize>> = HashMap::new();

    for (i, line) in lines.iter().enumerate() {
        let rack_key = line.rack_id.unwrap_or(0);

        rack_stops.entry(rack_key).or_insert_with(|| {
            let w = if line.rack_width == 0 { 50 } else { line.rack_width };
            let h = if line.rack_height == 0 { 50 } else { line.rack_height };
            PickStop {
                line_ids: vec![],
                rack_id: rack_key,
                rack_x: line.rack_x,
                rack_y: line.rack_y,
                center_x: line.rack_x as f64 + (w as f64) / 2.0,
                center_y: line.rack_y as f64 + (h as f64) / 2.0,
            }
        });

        rack_stops.get_mut(&rack_key).unwrap().line_ids.push(line.id);
        lines_by_rack.entry(rack_key).or_default().push(i);
    }

    let stops: Vec<PickStop> = rack_stops.into_values().collect();
    let route = calculate_route(&stops, 0, 0);

    // Assign sequences to lines based on route order
    let mut seq = 1;
    for stop in &route.stops {
        if let Some(indices) = lines_by_rack.get(&stop.rack_id) {
            for &idx in indices {
                lines[idx].sequence = seq;
            }
        }
        seq += 1;
    }

    lines.sort_by_key(|l| l.sequence);

    Ok(Json(RouteResponse {
        lines,
        route: Some(route),
    }))
}

// --- Internal helpers ---

/// Fetches and enriches move lines for a given picking ID.
/// Mirrors Go's `enrichPickLines` from `internal/handlers/picking.go`.
async fn enrich_pick_lines(
    db: &DatabaseConnection,
    picking_id: i64,
) -> Result<Vec<EnrichedPickLine>, (StatusCode, String)> {
    // Verify picking exists
    picking::Entity::find_by_id(picking_id)
        .one(db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Picking not found".to_string()))?;

    let move_lines = move_line::Entity::find()
        .filter(move_line::Column::PickingId.eq(picking_id))
        .all(db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Pre-load all racks for location→rack mapping
    let all_racks = rack::Entity::find().all(db).await.unwrap_or_default();

    let mut location_to_rack: HashMap<i64, &rack::Model> = HashMap::new();
    for r in &all_racks {
        if let Some(mapped_loc_id) = r.mapped_location_id {
            location_to_rack.insert(mapped_loc_id, r);
        }
    }

    let mut result = Vec::with_capacity(move_lines.len());

    for ml in move_lines {
        let mut el = EnrichedPickLine {
            id: ml.id,
            picking_id: ml.picking_id,
            product_id: ml.product_id,
            product_name: String::new(),
            product_barcode: String::new(),
            product_code: String::new(),
            qty_done: ml.qty_done,
            location_id: ml.location_id,
            location_name: String::new(),
            location_barcode: String::new(),
            rack_id: None,
            rack_name: None,
            rack_x: 0,
            rack_y: 0,
            rack_width: 0,
            rack_height: 0,
            state: ml.state,
            sequence: 0,
        };

        // Enrich product
        if let Ok(Some(prod)) = product::Entity::find_by_id(ml.product_id).one(db).await {
            el.product_name = prod.name;
            el.product_barcode = prod.barcode.0;
            el.product_code = prod.default_code.0;
        }

        // Enrich location + rack mapping
        if let Ok(Some(loc)) = location::Entity::find_by_id(ml.location_id).one(db).await {
            el.location_name = loc.complete_name.clone();
            el.location_barcode = loc.barcode.0.clone();

            // Find rack: check location itself, then parent
            let mut matched_rack = location_to_rack.get(&loc.id).copied();
            if matched_rack.is_none() {
                if let Some(parent_id) = loc.location_id {
                    matched_rack = location_to_rack.get(&parent_id).copied();
                }
            }

            if let Some(r) = matched_rack {
                el.rack_id = Some(r.id);
                el.rack_name = Some(r.name.clone());
                el.rack_x = r.pos_x;
                el.rack_y = r.pos_y;
                el.rack_width = if r.visual_width == 0 {
                    r.columns * 50
                } else {
                    r.visual_width
                };
                el.rack_height = if r.visual_height == 0 {
                    r.rows * 50
                } else {
                    r.visual_height
                };
            }
        }

        result.push(el);
    }

    Ok(result)
}
