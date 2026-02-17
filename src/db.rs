use sea_orm::{ConnectionTrait, Database, EntityTrait, PaginatorTrait, Schema};
pub use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::info;

use crate::ai::client::GeminiClient;
use crate::config::Config;
use crate::handlers::mesh_ws::MeshHub;
use crate::handlers::ws::WsHub;
use crate::models;
use crate::services::filestore::FileStoreService;
use crate::sync::engine::SyncEngine;
use crate::utils::identity::ServerIdentity;

/// Temporary state for a pairing session waiting for user approval
#[derive(Clone)]
pub struct PairingSession {
    pub code: String,
    pub remote_instance_id: String,
    pub remote_instance_name: String,
    pub remote_relay_url: String,
    pub created_at: std::time::Instant,
}

pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Config,
    pub sync_engine: SyncEngine,
    pub ai_client: Option<GeminiClient>,
    pub file_store: FileStoreService,
    pub ws_hub: WsHub,
    /// Mesh WebSocket hub for server-to-server signaling
    pub mesh_hub: MeshHub,
    /// Temporary setup password shown on login page when no users exist
    pub setup_password: Option<String>,
    /// Server Ed25519 identity for device pairing QR codes
    pub server_identity: ServerIdentity,
    /// Embedded PostgreSQL instance — must stay alive for the process lifetime
    pub _embedded_pg: Option<postgresql_embedded::PostgreSQL>,
    /// Active pairing sessions waiting for approval (keyed by code)
    pub pairing_sessions: Arc<RwLock<HashMap<String, PairingSession>>>,
}

pub async fn connect(database_url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    let mut opt = sea_orm::ConnectOptions::new(database_url);
    opt.max_connections(50)
        .min_connections(2)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8));

    Database::connect(opt).await
}

/// Start an embedded PostgreSQL instance. Returns the handle (must be kept alive) and the connection URL.
pub async fn start_embedded() -> Result<(postgresql_embedded::PostgreSQL, String), Box<dyn std::error::Error>> {
    use postgresql_embedded::Settings;

    info!("No DATABASE_URL set — starting embedded PostgreSQL...");

    let data_dir = PathBuf::from("./data/pg");
    std::fs::create_dir_all(&data_dir)?;

    let mut settings = Settings::default();
    settings.temporary = false;
    settings.data_dir = std::fs::canonicalize(&data_dir)?;
    settings.port = 5433;
    settings.username = "eckwms".to_string();
    settings.password = "eckwms".to_string();

    let mut pg = postgresql_embedded::PostgreSQL::new(settings);

    info!("Embedded PG: setting up (first run downloads ~100MB)...");
    pg.setup().await?;

    info!("Embedded PG: starting on port 5433...");
    pg.start().await?;

    let db_name = "eckwms";
    if !pg.database_exists(db_name).await? {
        info!("Embedded PG: creating database '{}'", db_name);
        pg.create_database(db_name).await?;
    }

    let url = pg.settings().url(db_name);
    info!("Embedded PG: ready at {}", url);

    Ok((pg, url))
}

/// Create all tables from sea-orm entity definitions (IF NOT EXISTS).
pub async fn create_schema(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);

    // Helper macro to avoid repetition
    macro_rules! create_table_if_not_exists {
        ($entity:path) => {
            let mut stmt = schema.create_table_from_entity($entity);
            db.execute(builder.build(
                stmt.if_not_exists()
            )).await?;
        };
    }

    info!("Creating schema tables (if not exists)...");
    create_table_if_not_exists!(models::user::Entity);
    create_table_if_not_exists!(models::product::Entity);
    create_table_if_not_exists!(models::product_alias::Entity);
    create_table_if_not_exists!(models::location::Entity);
    create_table_if_not_exists!(models::quant::Entity);
    create_table_if_not_exists!(models::checksum::Entity);
    create_table_if_not_exists!(models::picking::Entity);
    create_table_if_not_exists!(models::move_line::Entity);
    create_table_if_not_exists!(models::rack::Entity);
    create_table_if_not_exists!(models::partner::Entity);
    create_table_if_not_exists!(models::file_resource::Entity);
    create_table_if_not_exists!(models::attachment::Entity);
    create_table_if_not_exists!(models::delivery_carrier::Entity);
    create_table_if_not_exists!(models::stock_picking_delivery::Entity);
    create_table_if_not_exists!(models::delivery_tracking::Entity);
    create_table_if_not_exists!(models::sync_history::Entity);
    create_table_if_not_exists!(models::order::Entity);
    create_table_if_not_exists!(models::device_intake::Entity);
    create_table_if_not_exists!(models::inventory_discrepancy::Entity);
    create_table_if_not_exists!(models::document::Entity);
    create_table_if_not_exists!(models::mesh_node::Entity);
    create_table_if_not_exists!(models::registered_device::Entity);
    info!("Schema creation complete.");

    Ok(())
}

use crate::models::user;

/// If no users exist (or only the setup account exists), create/keep a temporary
/// setup account and return its password so it can be shown on the login page.
pub async fn seed_setup_account(db: &DatabaseConnection) -> Option<String> {
    use sea_orm::{ColumnTrait, QueryFilter};

    let count = user::Entity::find().count(db).await.unwrap_or(1);

    // Check if setup account already exists
    let setup_exists = user::Entity::find()
        .filter(user::Column::Email.eq("admin@setup.local"))
        .one(db)
        .await
        .ok()
        .flatten()
        .is_some();

    // If real users exist (not just setup), don't create/show setup account
    if count > 0 && !setup_exists {
        return None;
    }
    // If only setup account exists, regenerate password
    if setup_exists {
        let password: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(12)
            .map(char::from)
            .collect();
        let hash = bcrypt::hash(&password, 10).ok()?;
        // Update existing setup account password
        use sea_orm::sea_query::Expr;
        user::Entity::update_many()
            .filter(user::Column::Email.eq("admin@setup.local"))
            .col_expr(user::Column::Password, Expr::value(&hash))
            .exec(db)
            .await
            .ok()?;
        return Some(password);
    }
    if count > 0 {
        return None;
    }

    // Generate random 12-char password
    use rand::Rng;
    let password: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    let hash = bcrypt::hash(&password, 10).ok()?;

    use sea_orm::{ActiveModelTrait, Set};
    let new_user = user::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        username: Set("setup-admin".into()),
        password: Set(hash),
        email: Set("admin@setup.local".into()),
        name: Set(Some("Setup Admin".into())),
        role: Set("admin".into()),
        user_type: Set("individual".into()),
        company: Set(None),
        google_id: Set(None),
        pin: Set(String::new()),
        is_active: Set(true),
        last_login: Set(None),
        failed_login_attempts: Set(0),
        preferred_language: Set("en".into()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
    };

    if new_user.insert(db).await.is_ok() {
        tracing::info!("Created temporary setup account: admin@setup.local");
        Some(password)
    } else {
        None
    }
}
