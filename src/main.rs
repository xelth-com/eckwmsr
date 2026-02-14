mod config;
mod db;
mod handlers;
mod models;
mod utils;
mod web;

use axum::{routing::{get, post}, Router, Json};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, error};

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    server: String,
    version: String,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let cfg = config::load_config();
    info!("Starting eckWMS Rust Edition (eckwmsr)");
    info!("Instance ID: {}", cfg.instance_id);

    // Initialize Database
    let db_conn = match db::connect(&cfg.database_url).await {
        Ok(conn) => {
            info!("Database connection established");
            conn
        }
        Err(e) => {
            error!("Database connection failed: {}", e);
            std::process::exit(1);
        }
    };

    let app_state = Arc::new(db::AppState {
        db: db_conn,
        config: cfg.clone(),
    });

    // Build the router
    let app = Router::new()
        .route("/health", get(health_check))
        // Auth routes (support both root and /E subdirectory)
        .route("/auth/login", post(handlers::auth::login))
        .route("/E/auth/login", post(handlers::auth::login))
        // Fallback for static files (SPA frontend)
        .fallback(web::static_handler)
        .with_state(app_state);

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        server: "rust-local".to_string(),
        version: "0.1.0".to_string(),
    })
}
