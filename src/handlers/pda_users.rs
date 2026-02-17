use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{db::AppState, models::user};

/// Minimal user info returned to PDA user selector
#[derive(Serialize)]
pub struct UserSummary {
    pub id: String,
    pub username: String,
    pub name: String,
    pub role: String,
}

/// GET /api/users/active — list active users for PDA selector
pub async fn list_active_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserSummary>>, (StatusCode, String)> {
    let users = user::Entity::find()
        .filter(user::Column::IsActive.eq(true))
        .filter(user::Column::DeletedAt.is_null())
        .all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch users: {e}")))?;

    let result: Vec<UserSummary> = users
        .into_iter()
        .map(|u| UserSummary {
            id: u.id.to_string(),
            username: u.username,
            name: u.name.unwrap_or_default(),
            role: u.role,
        })
        .collect();

    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct VerifyPinRequest {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub pin: String,
}

/// POST /api/users/verify-pin — verify 4-digit PIN for PDA login
pub async fn verify_pin(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyPinRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if payload.user_id.is_empty() || payload.pin.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "userId and pin are required".to_string()));
    }

    let user_id: uuid::Uuid = payload.user_id.parse().map_err(|_| {
        (StatusCode::BAD_REQUEST, "Invalid user ID".to_string())
    })?;

    let user = user::Entity::find_by_id(user_id)
        .filter(user::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    if user.pin.is_empty() || user.pin != payload.pin {
        return Err((StatusCode::UNAUTHORIZED, "Wrong PIN".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "userId": user.id,
        "name": user.name,
    })))
}
