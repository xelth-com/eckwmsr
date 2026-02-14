use dotenvy::dotenv;
use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub instance_id: String,
    pub database_url: String,
    pub jwt_secret: String,
}

pub fn load_config() -> Config {
    // Ignore error if .env doesn't exist
    let _ = dotenv();

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3210".to_string())
        .parse()
        .unwrap_or(3210);

    let instance_id = env::var("INSTANCE_ID")
        .unwrap_or_else(|_| "rust_dev_node".to_string());

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/eckwms".to_string());

    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev_secret_change_me".to_string());

    Config {
        port,
        instance_id,
        database_url,
        jwt_secret,
    }
}
