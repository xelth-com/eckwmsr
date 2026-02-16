use sea_orm::{Database, EntityTrait, PaginatorTrait};
pub use sea_orm::DatabaseConnection;
use std::time::Duration;

use crate::ai::client::GeminiClient;
use crate::config::Config;
use crate::handlers::ws::WsHub;
use crate::services::filestore::FileStoreService;
use crate::sync::engine::SyncEngine;
use crate::utils::identity::ServerIdentity;

pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Config,
    pub sync_engine: SyncEngine,
    pub ai_client: Option<GeminiClient>,
    pub file_store: FileStoreService,
    pub ws_hub: WsHub,
    /// Temporary setup password shown on login page when no users exist
    pub setup_password: Option<String>,
    /// Server Ed25519 identity for device pairing QR codes
    pub server_identity: ServerIdentity,
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
