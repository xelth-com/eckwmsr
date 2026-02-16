use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, IntoActiveModel};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{db::AppState, models::user};

// --- DTOs ---

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub password: String,
    pub pin: Option<String>,
    #[serde(rename = "isActive")]
    pub is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub role: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub pin: Option<String>,
    #[serde(rename = "isActive")]
    pub is_active: Option<bool>,
}

#[derive(Serialize)]
pub struct SafeUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "hasPin")]
    pub has_pin: bool,
    #[serde(rename = "preferredLanguage")]
    pub preferred_language: String,
    #[serde(rename = "lastLogin", skip_serializing_if = "Option::is_none")]
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// --- Handlers ---

/// GET /api/admin/users
pub async fn list_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SafeUser>>, (StatusCode, String)> {
    let users = user::Entity::find()
        .filter(user::Column::DeletedAt.is_null())
        .order_by_desc(user::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let safe_users: Vec<SafeUser> = users
        .into_iter()
        .map(|u| SafeUser {
            id: u.id.to_string(),
            username: u.username,
            email: u.email,
            name: u.name,
            role: u.role,
            is_active: u.is_active,
            has_pin: !u.pin.is_empty(),
            preferred_language: u.preferred_language,
            last_login: u.last_login,
            created_at: u.created_at,
            updated_at: u.updated_at,
        })
        .collect();

    Ok(Json(safe_users))
}

/// POST /api/admin/users
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    if payload.username.is_empty() || payload.email.is_empty() || payload.password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "username, email and password are required".to_string(),
        ));
    }

    let hashed_password = bcrypt::hash(&payload.password, 10)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let role = if payload.role.is_empty() {
        "user".to_string()
    } else {
        payload.role
    };

    let new_user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(payload.username),
        email: Set(payload.email),
        name: Set(payload.name),
        role: Set(role),
        password: Set(hashed_password),
        pin: Set(payload.pin.unwrap_or_default()),
        is_active: Set(payload.is_active.unwrap_or(true)),
        user_type: Set("individual".to_string()),
        preferred_language: Set("en".to_string()),
        failed_login_attempts: Set(0),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    let inserted = new_user.insert(&state.db).await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("duplicate key") || msg.contains("unique") {
            (StatusCode::CONFLICT, "User already exists (check username/email)".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, msg)
        }
    })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": inserted.id,
            "username": inserted.username,
            "message": "User created"
        })),
    ))
}

/// PUT /api/admin/users/:id
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_model = user::Entity::find_by_id(id)
        .filter(user::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let mut active_user = user_model.into_active_model();

    if let Some(name) = payload.name {
        active_user.name = Set(Some(name));
    }
    if let Some(role) = payload.role {
        if !role.is_empty() {
            active_user.role = Set(role);
        }
    }
    if let Some(email) = payload.email {
        if !email.is_empty() {
            active_user.email = Set(email);
        }
    }
    if let Some(pin) = payload.pin {
        active_user.pin = Set(pin);
    }
    if let Some(is_active) = payload.is_active {
        active_user.is_active = Set(is_active);
    }
    if let Some(password) = payload.password {
        if !password.is_empty() {
            let hashed = bcrypt::hash(&password, 10)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            active_user.password = Set(hashed);
        }
    }
    active_user.updated_at = Set(chrono::Utc::now());

    active_user
        .update(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "id": id,
        "message": "User updated"
    })))
}

/// DELETE /api/admin/users/:id  (soft delete via deleted_at)
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_model = user::Entity::find_by_id(id)
        .filter(user::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let mut active_user = user_model.into_active_model();
    active_user.deleted_at = Set(Some(chrono::Utc::now()));
    active_user
        .update(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "User deleted"
    })))
}
