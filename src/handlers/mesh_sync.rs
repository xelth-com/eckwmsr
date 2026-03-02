use axum::{body::Bytes, extract::{Path, State}, http::{header, StatusCode}, Json, response::IntoResponse};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::AppState;
use crate::models::{attachment, document, file_resource, location, mesh_node, order, product, stock_picking_delivery, user};
use crate::sync::merkle_tree::{MerkleNode, MerkleRequest, MerkleTreeService};
use crate::sync::mesh_client::MeshClient;

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

/// Order representation for sync — includes deleted_at (skip_serializing on Model)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncableOrder {
    pub id: uuid::Uuid,
    pub order_number: String,
    pub order_type: String,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: String,
    pub item_id: Option<i32>,
    pub product_sku: String,
    pub product_name: String,
    pub serial_number: String,
    pub purchase_date: Option<chrono::DateTime<chrono::Utc>>,
    pub issue_description: String,
    pub diagnosis_notes: String,
    pub assigned_to: Option<String>,
    pub status: String,
    pub priority: String,
    pub repair_notes: String,
    pub parts_used: serde_json::Value,
    pub labor_hours: f64,
    pub total_cost: f64,
    pub resolution: String,
    pub notes: String,
    pub metadata: serde_json::Value,
    pub rma_reason: String,
    pub is_refund_requested: bool,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<order::Model> for SyncableOrder {
    fn from(o: order::Model) -> Self {
        Self {
            id: o.id,
            order_number: o.order_number,
            order_type: o.order_type,
            customer_name: o.customer_name,
            customer_email: o.customer_email,
            customer_phone: o.customer_phone,
            item_id: o.item_id,
            product_sku: o.product_sku,
            product_name: o.product_name,
            serial_number: o.serial_number,
            purchase_date: o.purchase_date,
            issue_description: o.issue_description,
            diagnosis_notes: o.diagnosis_notes,
            assigned_to: o.assigned_to,
            status: o.status,
            priority: o.priority,
            repair_notes: o.repair_notes,
            parts_used: o.parts_used,
            labor_hours: o.labor_hours,
            total_cost: o.total_cost,
            resolution: o.resolution,
            notes: o.notes,
            metadata: o.metadata,
            rma_reason: o.rma_reason,
            is_refund_requested: o.is_refund_requested,
            started_at: o.started_at,
            completed_at: o.completed_at,
            created_at: o.created_at,
            updated_at: o.updated_at,
            deleted_at: o.deleted_at,
        }
    }
}

/// Document representation for sync — includes deleted_at (skip_serializing on Model)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncableDocument {
    pub id: uuid::Uuid,
    pub r#type: String,
    pub status: String,
    pub payload: serde_json::Value,
    pub device_id: String,
    pub user_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<document::Model> for SyncableDocument {
    fn from(d: document::Model) -> Self {
        Self {
            id: d.id,
            r#type: d.r#type,
            status: d.status,
            payload: d.payload,
            device_id: d.device_id,
            user_id: d.user_id,
            created_at: d.created_at,
            updated_at: d.updated_at,
            deleted_at: d.deleted_at,
        }
    }
}

/// FileResource for sync — avatar_data encoded as base64 (Vec<u8> serializes as number array)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncableFileResource {
    pub id: uuid::Uuid,
    pub hash: String,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub width: i32,
    pub height: i32,
    pub avatar_data_b64: Option<String>,
    pub storage_path: String,
    pub created_by_device: String,
    pub context: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<file_resource::Model> for SyncableFileResource {
    fn from(f: file_resource::Model) -> Self {
        use base64::{engine::general_purpose::STANDARD, Engine};
        Self {
            id: f.id,
            hash: f.hash,
            original_name: f.original_name,
            mime_type: f.mime_type,
            size_bytes: f.size_bytes,
            width: f.width,
            height: f.height,
            avatar_data_b64: f.avatar_data.map(|bytes| STANDARD.encode(&bytes)),
            storage_path: f.storage_path,
            created_by_device: f.created_by_device,
            context: f.context,
            created_at: f.created_at,
            updated_at: f.updated_at,
            deleted_at: f.deleted_at,
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
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub orders: Vec<SyncableOrder>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub documents: Vec<SyncableDocument>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub file_resources: Vec<SyncableFileResource>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub attachments: Vec<attachment::Model>,
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
        orders: vec![],
        documents: vec![],
        file_resources: vec![],
        attachments: vec![],
    };

    match payload.entity_type.as_str() {
        "user" => {
            let parsed_uuids: Vec<uuid::Uuid> = payload.ids.iter().filter_map(|s| s.parse().ok()).collect();
            let mut query = user::Entity::find().filter(user::Column::Email.ne("admin@setup.local"));
            if !parsed_uuids.is_empty() {
                query = query.filter(user::Column::Id.is_in(parsed_uuids));
            }
            let users = query.all(&state.db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            resp.users = users.into_iter().map(SyncableUser::from).collect();
        }
        "order" => {
            let parsed_uuids: Vec<uuid::Uuid> = payload.ids.iter().filter_map(|s| s.parse().ok()).collect();
            let mut query = order::Entity::find();
            if !parsed_uuids.is_empty() {
                query = query.filter(order::Column::Id.is_in(parsed_uuids));
            }
            let orders = query.all(&state.db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            resp.orders = orders.into_iter().map(SyncableOrder::from).collect();
        }
        "document" => {
            let parsed_uuids: Vec<uuid::Uuid> = payload.ids.iter().filter_map(|s| s.parse().ok()).collect();
            let mut query = document::Entity::find();
            if !parsed_uuids.is_empty() {
                query = query.filter(document::Column::Id.is_in(parsed_uuids));
            }
            let docs = query.all(&state.db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            resp.documents = docs.into_iter().map(SyncableDocument::from).collect();
        }
        "file_resource" => {
            let parsed_uuids: Vec<uuid::Uuid> = payload.ids.iter().filter_map(|s| s.parse().ok()).collect();
            let mut query = file_resource::Entity::find();
            if !parsed_uuids.is_empty() {
                query = query.filter(file_resource::Column::Id.is_in(parsed_uuids));
            }
            let files = query.all(&state.db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            resp.file_resources = files.into_iter().map(SyncableFileResource::from).collect();
        }
        "attachment" => {
            let parsed_uuids: Vec<uuid::Uuid> = payload.ids.iter().filter_map(|s| s.parse().ok()).collect();
            let mut query = attachment::Entity::find();
            if !parsed_uuids.is_empty() {
                query = query.filter(attachment::Column::Id.is_in(parsed_uuids));
            }
            resp.attachments = query.all(&state.db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
        _ => {
            // Products, locations, shipments use i64 IDs; empty ids = return all
            let parsed_ids: Vec<i64> = payload
                .ids
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect();

            match payload.entity_type.as_str() {
                "product" => {
                    let mut query = product::Entity::find();
                    if !parsed_ids.is_empty() {
                        query = query.filter(product::Column::Id.is_in(parsed_ids));
                    }
                    resp.products = query
                        .all(&state.db)
                        .await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }
                "location" => {
                    let mut query = location::Entity::find();
                    if !parsed_ids.is_empty() {
                        query = query.filter(location::Column::Id.is_in(parsed_ids));
                    }
                    resp.locations = query
                        .all(&state.db)
                        .await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }
                "shipment" => {
                    let mut query = stock_picking_delivery::Entity::find();
                    if !parsed_ids.is_empty() {
                        query = query.filter(stock_picking_delivery::Column::Id.is_in(parsed_ids));
                    }
                    resp.shipments = query
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

#[derive(Clone, Serialize, Deserialize)]
pub struct PushPayload {
    #[serde(default)]
    pub products: Vec<product::Model>,
    #[serde(default)]
    pub locations: Vec<location::Model>,
    #[serde(default)]
    pub shipments: Vec<stock_picking_delivery::Model>,
    #[serde(default)]
    pub users: Vec<SyncableUser>,
    #[serde(default)]
    pub orders: Vec<SyncableOrder>,
    #[serde(default)]
    pub documents: Vec<SyncableDocument>,
    #[serde(default)]
    pub file_resources: Vec<SyncableFileResource>,
    #[serde(default)]
    pub attachments: Vec<attachment::Model>,
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

    // Upsert orders (mutable — status changes over time)
    for so in payload.orders {
        let am = order::ActiveModel {
            id: Set(so.id),
            order_number: Set(so.order_number),
            order_type: Set(so.order_type),
            customer_name: Set(so.customer_name),
            customer_email: Set(so.customer_email),
            customer_phone: Set(so.customer_phone),
            item_id: Set(so.item_id),
            product_sku: Set(so.product_sku),
            product_name: Set(so.product_name),
            serial_number: Set(so.serial_number),
            purchase_date: Set(so.purchase_date),
            issue_description: Set(so.issue_description),
            diagnosis_notes: Set(so.diagnosis_notes),
            assigned_to: Set(so.assigned_to),
            status: Set(so.status),
            priority: Set(so.priority),
            repair_notes: Set(so.repair_notes),
            parts_used: Set(so.parts_used),
            labor_hours: Set(so.labor_hours),
            total_cost: Set(so.total_cost),
            resolution: Set(so.resolution),
            notes: Set(so.notes),
            metadata: Set(so.metadata),
            rma_reason: Set(so.rma_reason),
            is_refund_requested: Set(so.is_refund_requested),
            started_at: Set(so.started_at),
            completed_at: Set(so.completed_at),
            created_at: Set(so.created_at),
            updated_at: Set(so.updated_at),
            deleted_at: Set(so.deleted_at),
        };
        let _ = order::Entity::insert(am)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(order::Column::Id)
                    .update_columns([
                        order::Column::CustomerName, order::Column::CustomerEmail,
                        order::Column::Status, order::Column::Priority,
                        order::Column::RepairNotes, order::Column::DiagnosisNotes,
                        order::Column::PartsUsed, order::Column::LaborHours,
                        order::Column::TotalCost, order::Column::Resolution,
                        order::Column::Notes, order::Column::Metadata,
                        order::Column::AssignedTo, order::Column::CompletedAt,
                        order::Column::UpdatedAt, order::Column::DeletedAt,
                    ])
                    .to_owned(),
            )
            .exec(db)
            .await;
        applied += 1;
    }

    // Upsert documents (immutable event log — skip if exists)
    for sd in payload.documents {
        let am = document::ActiveModel {
            id: Set(sd.id),
            r#type: Set(sd.r#type),
            status: Set(sd.status),
            payload: Set(sd.payload),
            device_id: Set(sd.device_id),
            user_id: Set(sd.user_id),
            created_at: Set(sd.created_at),
            updated_at: Set(sd.updated_at),
            deleted_at: Set(sd.deleted_at),
        };
        let _ = document::Entity::insert(am)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(document::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await;
        applied += 1;
    }

    // Upsert file_resources (CAS — immutable, skip if exists)
    for sf in payload.file_resources {
        use base64::{engine::general_purpose::STANDARD, Engine};
        let avatar_bytes = sf.avatar_data_b64.and_then(|b64| STANDARD.decode(&b64).ok());
        let am = file_resource::ActiveModel {
            id: Set(sf.id),
            hash: Set(sf.hash),
            original_name: Set(sf.original_name),
            mime_type: Set(sf.mime_type),
            size_bytes: Set(sf.size_bytes),
            width: Set(sf.width),
            height: Set(sf.height),
            avatar_data: Set(avatar_bytes),
            storage_path: Set(sf.storage_path),
            created_by_device: Set(sf.created_by_device),
            context: Set(sf.context),
            created_at: Set(sf.created_at),
            updated_at: Set(sf.updated_at),
            deleted_at: Set(sf.deleted_at),
        };
        let _ = file_resource::Entity::insert(am)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(file_resource::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await;
        applied += 1;
    }

    // Upsert attachments (linking record — immutable, skip if exists)
    for att in payload.attachments {
        let am = att.into_active_model();
        let _ = attachment::Entity::insert(am)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(attachment::Column::Id)
                    .do_nothing()
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

// --- PUSH TO ALL PEERS (generic helper) ---

/// Push a payload to all known mesh peers using 3-tier strategy.
/// Spawns a background task — does not block caller.
pub fn push_to_all_peers(state: Arc<AppState>, entity_type: &str, entity_id: &str, payload: PushPayload) {
    let entity_type = entity_type.to_string();
    let entity_id = entity_id.to_string();
    tokio::spawn(async move {
        let my_instance_id = state.config.instance_id.clone();
        let nodes = mesh_node::Entity::find()
            .all(&state.db)
            .await
            .unwrap_or_default();

        for node in nodes {
            if node.base_url.is_empty() { continue; }

            // 1) Direct HTTP push
            let client = MeshClient::new(&node.base_url);
            match client.push_entities(
                payload.products.clone(), payload.locations.clone(),
                payload.shipments.clone(), payload.users.clone(),
                payload.orders.clone(), payload.documents.clone(),
                payload.file_resources.clone(), payload.attachments.clone(),
            ).await {
                Ok(()) => {
                    tracing::info!("Direct push {} {} to {}", entity_type, entity_id, node.instance_id);
                    continue;
                }
                Err(e) => {
                    tracing::warn!("Direct push to {} failed: {}, trying fallbacks", node.instance_id, e);
                }
            }

            // 2) WebSocket signal
            if state.mesh_hub.is_peer_connected(&node.instance_id) {
                state.mesh_hub.notify_update(&my_instance_id, &entity_type, &entity_id);
                tracing::info!("WS signal for {} {} to {}", entity_type, entity_id, node.instance_id);
                continue;
            }

            // 3) Relay fallback (encrypt + push individual entity)
            let push_result = match entity_type.as_str() {
                "order" => {
                    if let Some(o) = payload.orders.first() {
                        state.sync_engine.push_entity(&node.instance_id, "order", &entity_id, o).await
                    } else { Ok(()) }
                }
                "document" => {
                    if let Some(d) = payload.documents.first() {
                        state.sync_engine.push_entity(&node.instance_id, "document", &entity_id, d).await
                    } else { Ok(()) }
                }
                "file_resource" => {
                    if let Some(f) = payload.file_resources.first() {
                        state.sync_engine.push_entity(&node.instance_id, "file_resource", &entity_id, f).await
                    } else { Ok(()) }
                }
                "attachment" => {
                    if let Some(a) = payload.attachments.first() {
                        state.sync_engine.push_entity(&node.instance_id, "attachment", &entity_id, a).await
                    } else { Ok(()) }
                }
                _ => Ok(()),
            };
            if let Err(e) = push_result {
                tracing::warn!("Relay push {} to {} failed: {}", entity_type, node.instance_id, e);
            }
        }
    });
}

// --- MESH FILE SERVING ---

/// GET /mesh/file/:hash — serves full CAS file content for mesh peers
pub async fn serve_mesh_file(
    State(state): State<Arc<AppState>>,
    Path(hash): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let file_res = file_resource::Entity::find()
        .filter(file_resource::Column::Hash.eq(&hash))
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    let content = state.file_store
        .get_file_content(&file_res)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    Ok((
        [
            (header::CONTENT_TYPE, file_res.mime_type),
            (header::CACHE_CONTROL, "public, max-age=31536000".to_string()),
        ],
        Bytes::from(content),
    ))
}
