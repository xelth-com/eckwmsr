use bcrypt::verify;
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use crate::models::user;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub email: String,
    pub role: String,
    #[serde(rename = "userType")]
    pub user_type: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub id: String,
    pub exp: usize,
}

/// Verify a plaintext password against a bcrypt hash (compatible with Go's bcrypt)
pub fn verify_password(password: &str, hash: &str) -> bool {
    verify(password, hash).unwrap_or(false)
}

/// Generate access + refresh JWT tokens matching the Go implementation
pub fn generate_tokens(user: &user::Model, secret: &str) -> Result<(String, String), String> {
    // Devices get 1-year tokens, others get 1 hour
    let is_device = user.role == "device" || user.user_type == "device";
    let exp_duration = if is_device {
        Duration::days(365)
    } else {
        Duration::hours(1)
    };

    let expiration = Utc::now() + exp_duration;

    let claims = Claims {
        id: user.id.to_string(),
        email: user.email.clone(),
        role: user.role.clone(),
        user_type: user.user_type.clone(),
        exp: expiration.timestamp() as usize,
    };

    let access_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| e.to_string())?;

    // Refresh token: 90 days (matches Go)
    let refresh_exp = Utc::now() + Duration::days(90);
    let refresh_claims = RefreshClaims {
        id: user.id.to_string(),
        exp: refresh_exp.timestamp() as usize,
    };

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| e.to_string())?;

    Ok((access_token, refresh_token))
}

/// Validate a JWT token and return claims
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, String> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| e.to_string())?;

    Ok(token_data.claims)
}
