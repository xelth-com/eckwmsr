use dotenvy::dotenv;
use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub instance_id: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub sync_relay_url: String,
    pub sync_network_key: String,
    pub gemini_api_key: String,
    pub gemini_primary_model: String,
    pub gemini_fallback_model: String,
}

pub fn load_config() -> Config {
    // Ignore error if .env doesn't exist
    let _ = dotenv();

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3210".to_string())
        .parse()
        .unwrap_or(3210);

    let instance_id =
        env::var("INSTANCE_ID").unwrap_or_else(|_| "rust_dev_node".to_string());

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/eckwms".to_string());

    let jwt_secret =
        env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret_change_me".to_string());

    let sync_relay_url =
        env::var("SYNC_RELAY_URL").unwrap_or_else(|_| "https://9eck.com".to_string());

    // 32-byte hex key (64 hex chars). Zeroed placeholder for dev.
    let sync_network_key = env::var("SYNC_NETWORK_KEY").unwrap_or_else(|_| {
        "0000000000000000000000000000000000000000000000000000000000000000".to_string()
    });

    let gemini_api_key = env::var("GEMINI_API_KEY").unwrap_or_default();
    let gemini_primary_model =
        env::var("GEMINI_PRIMARY_MODEL").unwrap_or_else(|_| "gemini-2.5-flash".to_string());
    let gemini_fallback_model =
        env::var("GEMINI_FALLBACK_MODEL").unwrap_or_else(|_| "gemini-2.0-flash".to_string());

    Config {
        port,
        instance_id,
        database_url,
        jwt_secret,
        sync_relay_url,
        sync_network_key,
        gemini_api_key,
        gemini_primary_model,
        gemini_fallback_model,
    }
}
