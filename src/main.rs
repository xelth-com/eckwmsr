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
    routing::{any, delete, get, post, put},
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

/// Compile-time timestamp (set by build.rs)
const BUILT_AT: &str = match option_env!("BUILT_AT") {
    Some(v) => v,
    None => "unknown",
};

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    server: String,
    version: String,
    built_at: String,
    started_at: String,
}

static STARTED_AT: std::sync::OnceLock<String> = std::sync::OnceLock::new();

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    STARTED_AT.get_or_init(|| chrono::Utc::now().to_rfc3339());

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

    // Ensure all tables exist (uses IF NOT EXISTS, safe for any database)
    if let Err(e) = db::create_schema(&db_conn).await {
        error!("Schema creation failed: {}", e);
        std::process::exit(1);
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
    let odoo_mutex = odoo_client.map(|c| tokio::sync::Mutex::new(c));

    // Initialize Twenty CRM client (optional)
    let twenty_client = if !cfg.twenty.url.is_empty() && !cfg.twenty.api_key.is_empty() {
        info!("Twenty CRM: Configured for {}", cfg.twenty.url);
        Some(services::twenty::TwentyClient::new(
            cfg.twenty.url.clone(),
            cfg.twenty.api_key.clone(),
        ))
    } else {
        info!("Twenty CRM: Not configured");
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

    // Bind the port BEFORE touching the DB — if port is taken, exit cleanly without
    // regenerating the setup password (which would desync the running server's in-memory
    // password from the newly written DB hash).
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Cannot bind {}: {} — is another instance already running?", addr, e);
            std::process::exit(1);
        }
    };
    info!("Server listening on {}", addr);

    // Seed setup account if no users exist
    let setup_password = Arc::new(tokio::sync::RwLock::new(
        db::seed_setup_account(&db_conn).await,
    ));
    if let Some(ref pw) = *setup_password.read().await {
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
        odoo_client: odoo_mutex,
        twenty_client,
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

    // Peer health check: every 30s for peers with base_url, relay fallback after 5 min.
    // Statuses: "active" (green) = direct OK, "degraded" (yellow) = direct failed but relay says online,
    //           "offline" (red) = both direct and relay say down.
    // Peers without base_url (behind NAT): always checked via relay.
    {
        let health_state = app_state.clone();
        tokio::spawn(async move {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap();
            let my_id = health_state.config.instance_id.clone();
            let relay_url = health_state.config.sync_relay_url.clone();
            let mesh_id = health_state.config.mesh_id.clone();
            // How long to wait before falling back to relay after direct ping fails
            const DEGRADED_GRACE_SECS: i64 = 300; // 5 minutes
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                use sea_orm::EntityTrait;
                let nodes: Vec<crate::models::mesh_node::Model> = crate::models::mesh_node::Entity::find()
                    .all(&health_state.db)
                    .await
                    .unwrap_or_default();

                // Check if any node needs relay lookup (no base_url, or degraded long enough)
                let needs_relay = nodes.iter().any(|n| {
                    if n.base_url.is_empty() { return true; }
                    // Also need relay if node was degraded for >5 min
                    if n.status == "degraded" {
                        let age = (chrono::Utc::now() - n.updated_at).num_seconds();
                        return age >= DEGRADED_GRACE_SECS;
                    }
                    false
                });

                // Fetch relay mesh status only when needed (avoid unnecessary calls)
                let relay_nodes: Vec<serde_json::Value> = if needs_relay {
                    match client
                        .get(format!("{}/E/mesh/{}/status", relay_url, mesh_id))
                        .send().await
                    {
                        Ok(resp) if resp.status().is_success() => {
                            resp.json::<serde_json::Value>().await.ok()
                                .and_then(|v| v.get("nodes").cloned())
                                .and_then(|n| serde_json::from_value(n).ok())
                                .unwrap_or_default()
                        }
                        _ => vec![],
                    }
                } else {
                    vec![]
                };

                let relay_online = |instance_id: &str| -> bool {
                    relay_nodes.iter().any(|rn| {
                        rn.get("instance_id").and_then(|id| id.as_str()) == Some(instance_id)
                            && rn.get("status").and_then(|s| s.as_str()) == Some("online")
                    })
                };

                for node in &nodes {
                    let new_status = if !node.base_url.is_empty() {
                        // Has base_url — try direct HTTP check first
                        let url = format!("{}/mesh/status?peer_id={}", node.base_url, my_id);
                        let direct_ok = match client.get(&url).send().await {
                            Ok(resp) if resp.status().is_success() => {
                                resp.json::<serde_json::Value>().await
                                    .map(|v| v.get("known").and_then(|k| k.as_bool()).unwrap_or(false))
                                    .unwrap_or(false)
                            }
                            _ => false,
                        };

                        if direct_ok {
                            "active" // green: direct ping OK
                        } else if node.status == "active" {
                            // Just failed — transition to degraded (yellow), start grace period
                            "degraded"
                        } else if node.status == "degraded" {
                            let age = (chrono::Utc::now() - node.updated_at).num_seconds();
                            if age >= DEGRADED_GRACE_SECS {
                                // Grace period over — ask relay
                                if relay_online(&node.instance_id) {
                                    "degraded" // relay says alive but direct unreachable
                                } else {
                                    "offline" // relay also says down → red
                                }
                            } else {
                                "degraded" // still waiting
                            }
                        } else {
                            // Was offline, still offline
                            if relay_online(&node.instance_id) {
                                "degraded" // relay says alive, maybe recovering
                            } else {
                                "offline"
                            }
                        }
                    } else {
                        // No base_url (behind NAT) — relay is the only source of truth
                        if relay_online(&node.instance_id) { "active" } else { "offline" }
                    };

                    let mut am: crate::models::mesh_node::ActiveModel = node.clone().into();
                    if new_status == "active" {
                        am.last_seen = sea_orm::Set(chrono::Utc::now());
                    }
                    // Only update updated_at on status transitions (for degraded timer)
                    if new_status != node.status {
                        am.updated_at = sea_orm::Set(chrono::Utc::now());
                    }
                    am.status = sea_orm::Set(new_status.to_string());
                    let _ = sea_orm::ActiveModelTrait::update(am, &health_state.db).await;
                }
            }
        });
    }
    info!("Peer health check started (every 60s, mutual verification)");

    // Outbound Mesh WebSocket Client
    {
        let ws_client_state = app_state.clone();
        tokio::spawn(async move {
            crate::sync::ws_client::start_outbound_ws_loop(ws_client_state).await;
        });
        info!("Outbound Mesh WS client loop started");
    }

    // Startup sync: pull all entity types from known mesh peers (fire-and-forget)
    {
        let startup_sync_state = app_state.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            use sea_orm::EntityTrait;
            let nodes = crate::models::mesh_node::Entity::find()
                .all(&startup_sync_state.db)
                .await
                .unwrap_or_default();
            for node in &nodes {
                if node.base_url.is_empty() { continue; }
                for entity_type in &["user", "order", "document", "file_resource", "attachment", "item", "order_item_event"] {
                    match startup_sync_state.sync_engine.full_pull_from_peer(&node.base_url, entity_type).await {
                        Ok(count) if count > 0 => {
                            info!("Startup sync: pulled {} {} from {}", count, entity_type, node.instance_id);
                            if *entity_type == "user" {
                                crate::db::cleanup_setup_if_real_users(
                                    &startup_sync_state.db,
                                    &startup_sync_state.setup_password,
                                ).await;
                            }
                        }
                        Ok(_) => {}
                        Err(e) => tracing::warn!("Startup sync {} with {} failed: {}", entity_type, node.instance_id, e),
                    }
                }
            }
        });
    }

    // Public API routes (no JWT — CAS files served via unguessable UUIDs)
    let public_api_routes = Router::new()
        .route("/files/:id", get(handlers::file::serve_file));

    // Protected API routes (require JWT)
    let protected_api_routes = Router::new()
        .route("/warehouse", get(handlers::warehouse::list_warehouses))
        .route("/items", get(handlers::warehouse::list_items))
        .route("/scan", post(handlers::scan::handle_scan))
        .route("/sync/trigger", post(handlers::sync::trigger_sync))
        .route("/sync/peers", post(handlers::sync::sync_with_peers))
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
        // Device status heartbeat (PDA calls this)
        .route("/status", get(handlers::device::device_status))
        // Device Pairing (QR code + admin management)
        .route("/internal/pairing-qr", get(handlers::device::generate_pairing_qr))
        .route("/admin/devices", get(handlers::device::list_devices))
        .route("/admin/devices/:id/status", put(handlers::device::update_device_status))
        .route("/admin/devices/:id/home", put(handlers::device::update_device_home))
        .route("/admin/devices/:id", delete(handlers::device::delete_device))
        .route("/admin/devices/:id/restore", post(handlers::device::restore_device))
        // PDA User API (multi-user selector + PIN verification)
        .route("/users/active", get(handlers::pda_users::list_active_users))
        .route("/users/verify-pin", post(handlers::pda_users::verify_pin))
        // Admin User Management
        .route("/admin/users", get(handlers::admin_users::list_users).post(handlers::admin_users::create_user))
        .route("/admin/users/:id", put(handlers::admin_users::update_user).delete(handlers::admin_users::delete_user))
        // Scraper management
        .route("/scraper/start", post(handlers::scraper_proxy::start_scraper))
        // Support API
        .route("/support/tickets", get(handlers::support::list_tickets))
        .route("/support/tickets/:ticket_id/threads", get(handlers::support::get_ticket_threads))
        .route("/support/tickets/:ticket_id/summary", post(handlers::support::summarize_ticket))
        .route("/support/import-thread", post(handlers::support::import_thread))
        // Analysis & Research
        .route("/analysis/support-dump", get(handlers::analysis::support_dump))
        // CRM write-back API
        .route("/crm/:entity_type/:id", put(handlers::crm::update_entity))
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
        .route("/push", post(handlers::mesh_sync::push_handler))
        .route("/file/:hash", get(handlers::mesh_sync::serve_mesh_file));

    // Scraper proxy (JWT-protected): /S/* → http://127.0.0.1:3211/*
    // Auth check is done inside the handler via the middleware injected via route_layer below.

    // Build the main router — strict /E prefix for microservice deployment
    let app = Router::new()
        // Health check (public)
        .route("/E/health", get(health_check))
        // WebSocket (public — uses device identify handshake)
        .route("/E/ws", get(handlers::ws::ws_handler))
        // Auth routes (public)
        .route("/E/auth/login", post(handlers::auth::login))
        .route("/E/auth/refresh", post(handlers::auth::refresh))
        .route("/E/auth/setup-status", get(handlers::auth::setup_status))
        // Device registration (public — uses Ed25519 signature, not JWT)
        .route("/E/api/internal/register-device", post(handlers::device::register_device))
        // Mesh routes (public)
        .nest("/E/mesh", mesh_routes)
        // API routes
        .nest("/E/api", api_routes)
        // RMA routes (root level, matching Go router)
        .nest("/E/rma", protected_rma_routes)
        // Scraper proxy: /E/S/* → http://127.0.0.1:3211/* (auth inside handler)
        .route("/E/S", any(handlers::scraper_proxy::proxy_handler))
        .route("/E/S/*path", any(handlers::scraper_proxy::proxy_handler))
        // Fallback for static files (SPA frontend)
        .fallback(web::static_handler)
        .with_state(app_state);

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
        server: "eckwmsr".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        built_at: BUILT_AT.to_string(),
        started_at: STARTED_AT.get().cloned().unwrap_or_default(),
    })
}
