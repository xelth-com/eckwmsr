use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::{db::AppState, utils::auth::validate_token};

/// JWT auth middleware — validates Bearer token and injects Claims into request extensions.
/// Mirrors Go's `middleware.AuthMiddleware` from `internal/middleware/auth.go`.
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let token = auth_header
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    match validate_token(token, &state.config.jwt_secret) {
        Ok(claims) => {
            // Insert claims into request extensions for handlers to use
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
