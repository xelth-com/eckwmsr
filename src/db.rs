use sea_orm::Database;
pub use sea_orm::DatabaseConnection;
use std::time::Duration;

use crate::config::Config;
use crate::sync::engine::SyncEngine;

pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Config,
    pub sync_engine: SyncEngine,
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
