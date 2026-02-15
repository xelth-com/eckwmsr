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
