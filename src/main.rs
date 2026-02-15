mod ai;
mod config;
mod db;
mod handlers;
mod middleware;
mod models;
mod services;
mod sync;
mod utils;
mod web;

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info};

use crate::sync::engine::SyncEngine;
use crate::sync::relay_client::RelayClient;
use crate::sync::security::{SecurityLayer, SyncNodeRole};

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    server: String,
    version: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cfg = config::load_config();
    info!("Starting eckWMS Rust Edition (eckwmsr)");
    info!("Instance ID: {}", cfg.instance_id);

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

    // Initialize Sync Engine
    let security_layer = SecurityLayer::new(SyncNodeRole::Peer, &cfg.sync_network_key);
    let relay_client = RelayClient::new(&cfg.sync_relay_url, &cfg.instance_id);
    let sync_engine = SyncEngine::new(
        db_conn.clone(),
        security_layer,
        relay_client,
        cfg.instance_id.clone(),
    );

    // Initialize AI Client (optional — runs without it if no API key)
    let ai_client = if !cfg.gemini_api_key.is_empty() {
        match ai::client::GeminiClient::new(
            &cfg.gemini_api_key,
            &cfg.gemini_primary_model,
            &cfg.gemini_fallback_model,
        ) {
            Ok(client) => Some(client),
            Err(e) => {
                error!("AI client initialization failed: {}", e);
                None
            }
        }
    } else {
        info!("AI: No GEMINI_API_KEY set, AI module disabled");
        None
    };

    // Initialize File Store (CAS)
    let file_store = services::filestore::FileStoreService::new(".");

    let app_state = Arc::new(db::AppState {
        db: db_conn,
        config: cfg.clone(),
        sync_engine,
        ai_client,
        file_store,
    });

    // Public API routes (no JWT — CAS files served via unguessable UUIDs)
    let public_api_routes = Router::new()
        .route("/files/:id", get(handlers::file::serve_file));

    // Protected API routes (require JWT)
    let protected_api_routes = Router::new()
        .route("/warehouse", get(handlers::warehouse::list_warehouses))
        .route("/items", get(handlers::warehouse::list_items))
        .route("/scan", post(handlers::scan::handle_scan))
        .route("/sync/trigger", post(handlers::sync::trigger_sync))
        .route("/sync/push_test", post(handlers::sync::trigger_push))
        // Pickings API
        .route("/pickings/active", get(handlers::picking::list_active_pickings))
        .route("/pickings/:id/lines", get(handlers::picking::get_picking_lines))
        .route("/pickings/:id/lines/:line_id/confirm", post(handlers::picking::confirm_pick_line))
        .route("/pickings/:id/validate", post(handlers::picking::validate_picking))
        .route("/pickings/:id/route", get(handlers::picking::get_picking_route))
        // AI API
        .route("/ai/analyze-image", post(handlers::ai::analyze_image))
        .route("/ai/respond", post(handlers::ai::handle_ai_respond))
        // Upload & Attachments
        .route("/upload/image", post(handlers::file::handle_image_upload))
        .route("/attachments/:model/:id", get(handlers::file::list_entity_attachments))
        .layer(from_fn_with_state(
            app_state.clone(),
            middleware::auth::auth_middleware,
        ));

    // Combine public + protected API routes
    let api_routes = Router::new()
        .merge(public_api_routes)
        .merge(protected_api_routes);

    // Build the main router — strict /E prefix for microservice deployment
    let app = Router::new()
        // Health check (public)
        .route("/E/health", get(health_check))
        // Auth routes (public)
        .route("/E/auth/login", post(handlers::auth::login))
        // API routes
        .nest("/E/api", api_routes)
        // Fallback for static files (SPA frontend)
        .fallback(web::static_handler)
        .with_state(app_state);

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
