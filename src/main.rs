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
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info};

use crate::sync::engine::SyncEngine;
use crate::sync::relay_client::RelayClient;
use crate::sync::security::{SecurityLayer, SyncNodeRole};
use crate::services::delivery::DeliveryService;
use crate::services::delivery_dhl::DhlProvider;
use crate::services::delivery_opal::OpalProvider;

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

    // Initialize Delivery Service
    let delivery_service = DeliveryService::new(db_conn.clone(), cfg.clone());

    // Register DHL provider if configured
    if let Ok(dhl_user) = std::env::var("DHL_USERNAME") {
        if !dhl_user.is_empty() {
            let dhl_pass = std::env::var("DHL_PASSWORD").unwrap_or_default();
            let dhl_url = std::env::var("DHL_URL").unwrap_or_default();
            delivery_service
                .register_provider(Box::new(DhlProvider::new(dhl_user, dhl_pass, dhl_url)))
                .await;
            info!("Registered DHL Delivery Provider");
        }
    }

    // Register OPAL provider if configured
    if let Ok(opal_user) = std::env::var("OPAL_USERNAME") {
        if !opal_user.is_empty() {
            let opal_pass = std::env::var("OPAL_PASSWORD").unwrap_or_default();
            let opal_url = std::env::var("OPAL_URL").unwrap_or_default();
            delivery_service
                .register_provider(Box::new(OpalProvider::new(opal_user, opal_pass, opal_url)))
                .await;
            info!("Registered OPAL Delivery Provider");
        }
    }

    // Drop delivery_service for now — it will be stored in AppState in Phase 8.3
    let _ = delivery_service;

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
        .route("/odoo/pickings", get(handlers::picking::list_odoo_pickings))
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
        // Delivery API
        .route("/delivery/config", get(handlers::delivery::get_delivery_config))
        .route("/delivery/shipments", get(handlers::delivery::list_shipments).post(handlers::delivery::create_shipment))
        .route("/delivery/shipments/:id", get(handlers::delivery::get_shipment))
        .route("/delivery/shipments/:id/cancel", post(handlers::delivery::cancel_shipment))
        .route("/delivery/import/opal", post(handlers::delivery::trigger_opal_import))
        .route("/delivery/import/dhl", post(handlers::delivery::trigger_dhl_import))
        .route("/delivery/carriers", get(handlers::delivery::list_carriers))
        .route("/delivery/sync/history", get(handlers::delivery::get_sync_history))
        // Print API
        .route("/print/labels", post(handlers::print::generate_labels))
        // Repair & Inventory API
        .route("/repair/event", post(handlers::repair::handle_repair_event))
        .route("/repair/events", get(handlers::repair::list_repair_events))
        .route("/inventory/discrepancies", get(handlers::inventory::list_discrepancies))
        .route("/inventory/discrepancies/stats", get(handlers::inventory::get_discrepancy_stats))
        .route("/inventory/discrepancies/:id", get(handlers::inventory::get_discrepancy))
        .route("/inventory/discrepancies/:id/review", put(handlers::inventory::review_discrepancy))
        .layer(from_fn_with_state(
            app_state.clone(),
            middleware::auth::auth_middleware,
        ));

    // Combine public + protected API routes
    let api_routes = Router::new()
        .merge(public_api_routes)
        .merge(protected_api_routes);

    // RMA/Orders routes (at /E/rma to match frontend api.js expectations)
    let protected_rma_routes = Router::new()
        .route("/", get(handlers::rma::list_orders).post(handlers::rma::create_order))
        .route("/:id", get(handlers::rma::get_order).put(handlers::rma::update_order).delete(handlers::rma::delete_order))
        .layer(from_fn_with_state(
            app_state.clone(),
            middleware::auth::auth_middleware,
        ));

    // Mesh routes (public — uses mesh tokens, not JWT)
    let mesh_routes = Router::new()
        .route("/nodes", get(handlers::mesh::list_nodes))
        .route("/status", get(handlers::mesh::get_status));

    // Build the main router — strict /E prefix for microservice deployment
    let app = Router::new()
        // Health check (public)
        .route("/E/health", get(health_check))
        // Auth routes (public)
        .route("/E/auth/login", post(handlers::auth::login))
        // Mesh routes (public)
        .nest("/E/mesh", mesh_routes)
        // API routes
        .nest("/E/api", api_routes)
        // RMA routes (root level, matching Go router)
        .nest("/E/rma", protected_rma_routes)
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
