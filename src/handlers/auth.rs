use axum::{extract::State, http::StatusCode, Json};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::{db::AppState, models::user, utils::auth};

#[derive(Deserialize)]
pub struct LoginRequest {
    #[serde(alias = "Email")]
    pub email: String,
    #[serde(alias = "Password")]
    pub password: String,
}

#[derive(Serialize)]
pub struct Tokens {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub tokens: Tokens,
    pub user: user::Model,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

/// GET /auth/setup-status — returns setup credentials if this is a fresh install
pub async fn setup_status(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    match &*state.setup_password.read().await {
        Some(pw) => Json(serde_json::json!({
            "needsSetup": true,
            "email": "admin@setup.local",
            "password": pw
        })),
        None => Json(serde_json::json!({
            "needsSetup": false
        })),
    }
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("[AUTH] Login attempt for: {}", payload.email);

    // Find user by email or username (matches Go: tries username first, then email)
    let found_user = user::Entity::find()
        .filter(
            sea_orm::Condition::any()
                .add(user::Column::Username.eq(&payload.email))
                .add(user::Column::Email.eq(&payload.email)),
        )
        .one(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?
        .ok_or_else(|| {
            info!("[AUTH] User not found: {}", payload.email);
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid credentials".to_string(),
                }),
            )
        })?;

    // Verify bcrypt password hash (compatible with Go's bcrypt.GenerateFromPassword)
    if !auth::verify_password(&payload.password, &found_user.password) {
        info!("[AUTH] Password mismatch for user: {}", payload.email);
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid credentials".to_string(),
            }),
        ));
    }

    info!("[AUTH] Login successful for user: {}", found_user.email);

    // Generate JWT tokens (HS256, same as Go)
    let (access_token, refresh_token) =
        auth::generate_tokens(&found_user, &state.config.jwt_secret).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to generate tokens: {}", e),
                }),
            )
        })?;

    Ok(Json(LoginResponse {
        tokens: Tokens {
            access_token,
            refresh_token,
        },
        user: found_user,
    }))
}

/// POST /auth/refresh — exchanges a valid refresh token for a new token pair
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let claims = auth::validate_refresh_token(&payload.refresh_token, &state.config.jwt_secret)
        .map_err(|e| {
            info!("[AUTH] Invalid refresh token: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse { error: "Invalid or expired refresh token".into() }),
            )
        })?;

    let user_id = uuid::Uuid::parse_str(&claims.id).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Invalid user ID format in token".into() }))
    })?;

    let found_user = user::Entity::find_by_id(user_id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?
        .ok_or_else(|| {
            (StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "User not found".into() }))
        })?;

    if !found_user.is_active || found_user.deleted_at.is_some() {
        return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "User account is disabled".into() })));
    }

    let (access_token, refresh_token) = auth::generate_tokens(&found_user, &state.config.jwt_secret)
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: format!("Failed to generate tokens: {}", e) }))
        })?;

    info!("[AUTH] Token refreshed for user: {}", found_user.email);

    Ok(Json(LoginResponse {
        tokens: Tokens { access_token, refresh_token },
        user: found_user,
    }))
}
