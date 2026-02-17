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
use crate::services::odoo::OdooClient;

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

    // Database: use external PostgreSQL if DATABASE_URL is set, otherwise start embedded PG
    let (embedded_pg, database_url) = if cfg.database_url.is_empty() {
        match db::start_embedded().await {
            Ok((pg, url)) => (Some(pg), url),
            Err(e) => {
                error!("Embedded PostgreSQL failed to start: {}", e);
                error!("Set DATABASE_URL to use an external database instead.");
                std::process::exit(1);
            }
        }
    } else {
        info!("Using external database: {}", cfg.database_url);
        (None, cfg.database_url.clone())
    };

    let db_conn = match db::connect(&database_url).await {
        Ok(conn) => {
            info!("Database connection established");
            conn
        }
        Err(e) => {
            error!("Database connection failed: {}", e);
            std::process::exit(1);
        }
    };

    // If using embedded PG, ensure all tables exist
    if embedded_pg.is_some() {
        if let Err(e) = db::create_schema(&db_conn).await {
            error!("Schema creation failed: {}", e);
            std::process::exit(1);
        }
    }

    // Initialize Sync Engine
    let security_layer = SecurityLayer::new(SyncNodeRole::Peer, &cfg.sync_network_key);
    let relay_client = RelayClient::new(&cfg.sync_relay_url, &cfg.instance_id, &cfg.mesh_id);
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

    // Initialize Odoo Client (optional — runs without it if not configured)
    let odoo_client = if !cfg.odoo.url.is_empty() && !cfg.odoo.username.is_empty() {
        info!("Odoo: Configured for {}", cfg.odoo.url);
        Some(OdooClient::new(
            cfg.odoo.url.clone(),
            cfg.odoo.database.clone(),
            cfg.odoo.username.clone(),
            cfg.odoo.password.clone(),
        ))
    } else {
        info!("Odoo: Not configured, repair sync disabled");
        None
    };
    let _ = odoo_client; // Will be wired into RepairService/AppState in a future step

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

    // Seed setup account if no users exist
    let setup_password = db::seed_setup_account(&db_conn).await;
    if let Some(ref pw) = setup_password {
        info!("=================================================");
        info!("  FIRST RUN: Setup account created");
        info!("  Email: admin@setup.local");
        info!("  Password: {}", pw);
        info!("  Create your own account, then this one will be removed.");
        info!("=================================================");
    }

    let ws_hub = handlers::ws::WsHub::new();
    let mesh_hub = handlers::mesh_ws::MeshHub::new();

    // Initialize server identity for device pairing (Ed25519 keypair)
    let server_identity = utils::identity::load_or_generate_identity(&cfg.instance_id);
    info!("Server identity loaded, public key: {}...", &server_identity.public_key[..8]);

    // Create a relay client for the heartbeat task
    let heartbeat_relay = RelayClient::new(&cfg.sync_relay_url, &cfg.instance_id, &cfg.mesh_id);
    let heartbeat_base_url = cfg.base_url.clone();
    let heartbeat_port = cfg.port;

    let app_state = Arc::new(db::AppState {
        db: db_conn,
        config: cfg.clone(),
        sync_engine,
        ai_client,
        file_store,
        ws_hub,
        mesh_hub,
        setup_password,
        server_identity,
        _embedded_pg: embedded_pg,
        pairing_sessions: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    });

    // Start heartbeat background task (every 5 minutes)
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            let (ip, port) = parse_base_url(&heartbeat_base_url, heartbeat_port);
            match heartbeat_relay.send_heartbeat(&ip, port, None).await {
                Ok(_) => {}
                Err(e) => tracing::warn!("Heartbeat failed: {}", e),
            }
        }
    });
    info!("Heartbeat task started (every 5 min), mesh_id: {}", cfg.mesh_id);

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
        // Pairing API (S2S Magic Code)
        .route("/pairing/host", post(handlers::pairing::host_pairing))
        .route("/pairing/connect", post(handlers::pairing::join_pairing))
        .route("/pairing/check", post(handlers::pairing::check_pairing))
        .route("/pairing/approve", post(handlers::pairing::approve_pairing))
        .route("/pairing/finalize", post(handlers::pairing::finalize_pairing))
        // Device Pairing (QR code + admin management)
        .route("/internal/pairing-qr", get(handlers::device::generate_pairing_qr))
        .route("/admin/devices", get(handlers::device::list_devices))
        .route("/admin/devices/:id/status", put(handlers::device::update_device_status))
        .route("/admin/devices/:id/home", put(handlers::device::update_device_home))
        .route("/admin/devices/:id", delete(handlers::device::delete_device))
        .route("/admin/devices/:id/restore", post(handlers::device::restore_device))
        // Admin User Management
        .route("/admin/users", get(handlers::admin_users::list_users).post(handlers::admin_users::create_user))
        .route("/admin/users/:id", put(handlers::admin_users::update_user).delete(handlers::admin_users::delete_user))
        // Admin Config & Mesh Management
        .route("/admin/config/save-key", post(handlers::config::save_network_key))
        .route("/admin/mesh/:id", delete(handlers::mesh::delete_node))
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
        .route("/status", get(handlers::mesh::get_status))
        .route("/relay-status", get(handlers::mesh::get_relay_status))
        .route("/ws", get(handlers::mesh_ws::mesh_ws_handler))
        .route("/merkle", post(handlers::mesh_sync::merkle_handler))
        .route("/pull", post(handlers::mesh_sync::pull_handler))
        .route("/push", post(handlers::mesh_sync::push_handler));

    // Build the main router — strict /E prefix for microservice deployment
    let app = Router::new()
        // Health check (public)
        .route("/E/health", get(health_check))
        // WebSocket (public — uses device identify handshake)
        .route("/E/ws", get(handlers::ws::ws_handler))
        // Auth routes (public)
        .route("/E/auth/login", post(handlers::auth::login))
        .route("/E/auth/setup-status", get(handlers::auth::setup_status))
        // Device registration (public — uses Ed25519 signature, not JWT)
        .route("/E/api/internal/register-device", post(handlers::device::register_device))
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

/// Extract IP and port from BASE_URL (e.g. "https://pda.repair" or "http://192.168.1.50:3210")
fn parse_base_url(base_url: &str, default_port: u16) -> (String, u16) {
    if base_url.is_empty() {
        return ("0.0.0.0".to_string(), default_port);
    }
    let url = base_url
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    if let Some(colon_pos) = url.rfind(':') {
        let ip = &url[..colon_pos];
        let port = url[colon_pos + 1..].parse().unwrap_or(default_port);
        (ip.to_string(), port)
    } else {
        (url.to_string(), default_port)
    }
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        server: "rust-local".to_string(),
        version: "0.1.0".to_string(),
    })
}
