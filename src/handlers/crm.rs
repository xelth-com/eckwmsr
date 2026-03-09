use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::db::AppState;

/// PUT /api/crm/:entity_type/:id — update a Twenty CRM entity
pub async fn update_entity(
    State(state): State<Arc<AppState>>,
    Path((entity_type, id)): Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let client = state.twenty_client.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "Twenty CRM integration not configured".to_string(),
    ))?;

    let result = match entity_type.as_str() {
        "company" => client.update_company(&id, &payload).await,
        "person" => client.update_person(&id, &payload).await,
        "opportunity" => client.update_opportunity(&id, &payload).await,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Unsupported CRM entity: {}", entity_type),
            ))
        }
    };

    match result {
        Ok(data) => Ok(Json(data)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
