use axum::{extract::State, http::StatusCode, Json};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::AppState;
use crate::models::{location, product, stock_picking_delivery, user};
use crate::sync::merkle_tree::{MerkleNode, MerkleRequest, MerkleTreeService};

/// User representation that includes all fields for sync (password hash, pin, deleted_at).
/// The normal user::Model skips these in serialization for API safety.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncableUser {
    pub id: uuid::Uuid,
    pub username: String,
    pub password: String,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub user_type: String,
    pub company: Option<String>,
    pub google_id: Option<String>,
    pub pin: String,
    pub is_active: bool,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub failed_login_attempts: i64,
    pub preferred_language: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<user::Model> for SyncableUser {
    fn from(u: user::Model) -> Self {
        Self {
            id: u.id,
            username: u.username,
            password: u.password,
            email: u.email,
            name: u.name,
            role: u.role,
            user_type: u.user_type,
            company: u.company,
            google_id: u.google_id,
            pin: u.pin,
            is_active: u.is_active,
            last_login: u.last_login,
            failed_login_attempts: u.failed_login_attempts,
            preferred_language: u.preferred_language,
            created_at: u.created_at,
            updated_at: u.updated_at,
            deleted_at: u.deleted_at,
        }
    }
}

// --- MERKLE ---

/// POST /mesh/merkle — Get Merkle tree state for comparison
pub async fn merkle_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MerkleRequest>,
) -> Result<Json<MerkleNode>, (StatusCode, String)> {
    let svc = MerkleTreeService::new(&state.db);
    let node = svc
        .get_state(&payload)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(node))
}

// --- PULL ---

#[derive(Serialize, Deserialize)]
pub struct PullRequest {
    pub entity_type: String,
    pub ids: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PullResponse {
    pub entity_type: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub products: Vec<product::Model>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub locations: Vec<location::Model>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub shipments: Vec<stock_picking_delivery::Model>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub users: Vec<SyncableUser>,
}

/// POST /mesh/pull — Fetch specific entities by ID for sync
pub async fn pull_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PullRequest>,
) -> Result<Json<PullResponse>, (StatusCode, String)> {
    let mut resp = PullResponse {
        entity_type: payload.entity_type.clone(),
        products: vec![],
        locations: vec![],
        shipments: vec![],
        users: vec![],
    };

    match payload.entity_type.as_str() {
        "user" => {
            // Parse UUIDs for user IDs
            let parsed_uuids: Vec<uuid::Uuid> = payload
                .ids
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect();
            let users = user::Entity::find()
                .filter(user::Column::Id.is_in(parsed_uuids))
                .filter(user::Column::Email.ne("admin@setup.local"))
                .all(&state.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            resp.users = users.into_iter().map(SyncableUser::from).collect();
        }
        _ => {
            // Products, locations, shipments use i64 IDs
            let parsed_ids: Vec<i64> = payload
                .ids
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect();

            match payload.entity_type.as_str() {
                "product" => {
                    resp.products = product::Entity::find()
                        .filter(product::Column::Id.is_in(parsed_ids))
                        .all(&state.db)
                        .await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }
                "location" => {
                    resp.locations = location::Entity::find()
                        .filter(location::Column::Id.is_in(parsed_ids))
                        .all(&state.db)
                        .await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }
                "shipment" => {
                    resp.shipments = stock_picking_delivery::Entity::find()
                        .filter(stock_picking_delivery::Column::Id.is_in(parsed_ids))
                        .all(&state.db)
                        .await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }
                other => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        format!("Unknown entity_type: {}", other),
                    ));
                }
            }
        }
    }

    Ok(Json(resp))
}

// --- PUSH ---

#[derive(Serialize, Deserialize)]
pub struct PushPayload {
    #[serde(default)]
    pub products: Vec<product::Model>,
    #[serde(default)]
    pub locations: Vec<location::Model>,
    #[serde(default)]
    pub shipments: Vec<stock_picking_delivery::Model>,
    #[serde(default)]
    pub users: Vec<SyncableUser>,
}

/// POST /mesh/push — Apply incoming entities (upsert)
pub async fn push_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PushPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let db = &state.db;
    let mut applied = 0u32;

    for p in payload.products {
        let am = p.into_active_model();
        let _ = product::Entity::insert(am)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(product::Column::Id)
                    .update_columns([
                        product::Column::DefaultCode,
                        product::Column::Barcode,
                        product::Column::Name,
                        product::Column::Active,
                        product::Column::Type,
                        product::Column::ListPrice,
                        product::Column::StandardPrice,
                        product::Column::Weight,
                        product::Column::Volume,
                        product::Column::WriteDate,
                        product::Column::LastSyncedAt,
                    ])
                    .to_owned(),
            )
            .exec(db)
            .await;
        applied += 1;
    }

    for l in payload.locations {
        let am = l.into_active_model();
        let _ = location::Entity::insert(am)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(location::Column::Id)
                    .update_columns([
                        location::Column::Name,
                        location::Column::CompleteName,
                        location::Column::Barcode,
                        location::Column::Usage,
                        location::Column::LocationId,
                        location::Column::Active,
                        location::Column::LastSyncedAt,
                        location::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(db)
            .await;
        applied += 1;
    }

    for s in payload.shipments {
        let am = s.into_active_model();
        let _ = stock_picking_delivery::Entity::insert(am)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(stock_picking_delivery::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await;
        applied += 1;
    }

    // Upsert synced users (skip setup account)
    for su in payload.users {
        if su.email == "admin@setup.local" {
            continue;
        }
        let am = user::ActiveModel {
            id: Set(su.id),
            username: Set(su.username),
            password: Set(su.password),
            email: Set(su.email),
            name: Set(su.name),
            role: Set(su.role),
            user_type: Set(su.user_type),
            company: Set(su.company),
            google_id: Set(su.google_id),
            pin: Set(su.pin),
            is_active: Set(su.is_active),
            last_login: Set(su.last_login),
            failed_login_attempts: Set(su.failed_login_attempts),
            preferred_language: Set(su.preferred_language),
            created_at: Set(su.created_at),
            updated_at: Set(su.updated_at),
            deleted_at: Set(su.deleted_at),
        };
        let _ = user::Entity::insert(am)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(user::Column::Id)
                    .update_columns([
                        user::Column::Username,
                        user::Column::Password,
                        user::Column::Email,
                        user::Column::Name,
                        user::Column::Role,
                        user::Column::UserType,
                        user::Column::Pin,
                        user::Column::IsActive,
                        user::Column::PreferredLanguage,
                        user::Column::UpdatedAt,
                        user::Column::DeletedAt,
                    ])
                    .to_owned(),
            )
            .exec(db)
            .await;
        applied += 1;
    }

    // If any users were synced, clean up the setup account
    if applied > 0 {
        crate::db::cleanup_setup_if_real_users(db, &state.setup_password).await;
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "applied": applied
    })))
}
