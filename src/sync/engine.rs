use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{sea_query::OnConflict, IntoActiveModel};
use serde::Serialize;
use tracing::{error, info, warn};

use std::collections::BTreeMap;

use crate::models::sync_packet::EntityMetadata;
use crate::models::{attachment, document, file_resource, item, location, order, order_item_event, product, stock_picking_delivery, user};
use crate::handlers::mesh_sync::{SyncableUser, SyncableOrder, SyncableDocument, SyncableFileResource};
use crate::sync::mesh_client::MeshClient;
use crate::sync::merkle_tree::{compare_trees, MerkleRequest, MerkleTreeService};
use crate::sync::relay_client::RelayClient;
use crate::sync::security::SecurityLayer;
use crate::sync::vector_clock::VectorClock;

pub struct SyncEngine {
    db: DatabaseConnection,
    security: SecurityLayer,
    relay: RelayClient,
    instance_id: String,
}

impl SyncEngine {
    pub fn new(
        db: DatabaseConnection,
        security: SecurityLayer,
        relay: RelayClient,
        instance_id: String,
    ) -> Self {
        Self {
            db,
            security,
            relay,
            instance_id,
        }
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn relay_client(&self) -> RelayClient {
        self.relay.clone()
    }

    // --- Shared upsert methods (used by both Relay and Mesh paths) ---

    async fn upsert_product(&self, p: product::Model) -> Result<(), DbErr> {
        let am = p.into_active_model();
        product::Entity::insert(am)
            .on_conflict(
                OnConflict::column(product::Column::Id)
                    .update_columns([
                        product::Column::Name,
                        product::Column::Barcode,
                        product::Column::DefaultCode,
                        product::Column::Active,
                        product::Column::ListPrice,
                        product::Column::StandardPrice,
                        product::Column::Weight,
                        product::Column::Volume,
                        product::Column::Type,
                        product::Column::WriteDate,
                        product::Column::LastSyncedAt,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await?;
        Ok(())
    }

    async fn upsert_location(&self, l: location::Model) -> Result<(), DbErr> {
        let am = l.into_active_model();
        location::Entity::insert(am)
            .on_conflict(
                OnConflict::column(location::Column::Id)
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
            .exec(&self.db)
            .await?;
        Ok(())
    }

    async fn upsert_shipment(&self, s: stock_picking_delivery::Model) -> Result<(), DbErr> {
        let am = s.into_active_model();
        let _ = stock_picking_delivery::Entity::insert(am)
            .on_conflict(
                OnConflict::column(stock_picking_delivery::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&self.db)
            .await;
        Ok(())
    }

    async fn upsert_user(&self, su: SyncableUser) -> Result<(), DbErr> {
        if su.email == "admin@setup.local" {
            return Ok(()); // Never sync the temporary setup account
        }
        let am = user::ActiveModel {
            id: sea_orm::Set(su.id),
            username: sea_orm::Set(su.username),
            password: sea_orm::Set(su.password),
            email: sea_orm::Set(su.email),
            name: sea_orm::Set(su.name),
            role: sea_orm::Set(su.role),
            user_type: sea_orm::Set(su.user_type),
            company: sea_orm::Set(su.company),
            google_id: sea_orm::Set(su.google_id),
            pin: sea_orm::Set(su.pin),
            is_active: sea_orm::Set(su.is_active),
            last_login: sea_orm::Set(su.last_login),
            failed_login_attempts: sea_orm::Set(su.failed_login_attempts),
            preferred_language: sea_orm::Set(su.preferred_language),
            created_at: sea_orm::Set(su.created_at),
            updated_at: sea_orm::Set(su.updated_at),
            deleted_at: sea_orm::Set(su.deleted_at),
        };
        user::Entity::insert(am)
            .on_conflict(
                OnConflict::column(user::Column::Id)
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
            .exec(&self.db)
            .await?;
        Ok(())
    }

    async fn upsert_order(&self, so: SyncableOrder) -> Result<(), DbErr> {
        let am = order::ActiveModel {
            id: sea_orm::Set(so.id),
            order_number: sea_orm::Set(so.order_number),
            order_type: sea_orm::Set(so.order_type),
            customer_name: sea_orm::Set(so.customer_name),
            customer_email: sea_orm::Set(so.customer_email),
            customer_phone: sea_orm::Set(so.customer_phone),
            item_id: sea_orm::Set(so.item_id),
            product_sku: sea_orm::Set(so.product_sku),
            product_name: sea_orm::Set(so.product_name),
            serial_number: sea_orm::Set(so.serial_number),
            purchase_date: sea_orm::Set(so.purchase_date),
            issue_description: sea_orm::Set(so.issue_description),
            diagnosis_notes: sea_orm::Set(so.diagnosis_notes),
            assigned_to: sea_orm::Set(so.assigned_to),
            status: sea_orm::Set(so.status),
            priority: sea_orm::Set(so.priority),
            repair_notes: sea_orm::Set(so.repair_notes),
            parts_used: sea_orm::Set(so.parts_used),
            labor_hours: sea_orm::Set(so.labor_hours),
            total_cost: sea_orm::Set(so.total_cost),
            resolution: sea_orm::Set(so.resolution),
            notes: sea_orm::Set(so.notes),
            metadata: sea_orm::Set(so.metadata),
            rma_reason: sea_orm::Set(so.rma_reason),
            is_refund_requested: sea_orm::Set(so.is_refund_requested),
            started_at: sea_orm::Set(so.started_at),
            completed_at: sea_orm::Set(so.completed_at),
            created_at: sea_orm::Set(so.created_at),
            updated_at: sea_orm::Set(so.updated_at),
            deleted_at: sea_orm::Set(so.deleted_at),
        };
        order::Entity::insert(am)
            .on_conflict(
                OnConflict::column(order::Column::Id)
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
            .exec(&self.db)
            .await?;
        Ok(())
    }

    async fn upsert_document(&self, sd: SyncableDocument) -> Result<(), DbErr> {
        let am = document::ActiveModel {
            id: sea_orm::Set(sd.id),
            r#type: sea_orm::Set(sd.r#type),
            status: sea_orm::Set(sd.status),
            payload: sea_orm::Set(sd.payload),
            device_id: sea_orm::Set(sd.device_id),
            user_id: sea_orm::Set(sd.user_id),
            created_at: sea_orm::Set(sd.created_at),
            updated_at: sea_orm::Set(sd.updated_at),
            deleted_at: sea_orm::Set(sd.deleted_at),
        };
        let _ = document::Entity::insert(am)
            .on_conflict(OnConflict::column(document::Column::Id).do_nothing().to_owned())
            .exec(&self.db)
            .await;
        Ok(())
    }

    async fn upsert_file_resource(&self, sf: SyncableFileResource) -> Result<(), DbErr> {
        use base64::{engine::general_purpose::STANDARD, Engine};
        let avatar_bytes = sf.avatar_data_b64.and_then(|b64| STANDARD.decode(&b64).ok());
        let am = file_resource::ActiveModel {
            id: sea_orm::Set(sf.id),
            hash: sea_orm::Set(sf.hash),
            original_name: sea_orm::Set(sf.original_name),
            mime_type: sea_orm::Set(sf.mime_type),
            size_bytes: sea_orm::Set(sf.size_bytes),
            width: sea_orm::Set(sf.width),
            height: sea_orm::Set(sf.height),
            avatar_data: sea_orm::Set(avatar_bytes),
            storage_path: sea_orm::Set(sf.storage_path),
            created_by_device: sea_orm::Set(sf.created_by_device),
            context: sea_orm::Set(sf.context),
            created_at: sea_orm::Set(sf.created_at),
            updated_at: sea_orm::Set(sf.updated_at),
            deleted_at: sea_orm::Set(sf.deleted_at),
        };
        let _ = file_resource::Entity::insert(am)
            .on_conflict(OnConflict::column(file_resource::Column::Id).do_nothing().to_owned())
            .exec(&self.db)
            .await;
        Ok(())
    }

    async fn upsert_attachment(&self, att: attachment::Model) -> Result<(), DbErr> {
        let am = att.into_active_model();
        let _ = attachment::Entity::insert(am)
            .on_conflict(OnConflict::column(attachment::Column::Id).do_nothing().to_owned())
            .exec(&self.db)
            .await;
        Ok(())
    }

    async fn upsert_item(&self, it: item::Model) -> Result<(), DbErr> {
        let am = it.into_active_model();
        let _ = item::Entity::insert(am)
            .on_conflict(
                OnConflict::column(item::Column::Id)
                    .update_columns([
                        item::Column::PrimaryBarcode,
                        item::Column::Barcodes,
                        item::Column::Name,
                        item::Column::MainPhotoId,
                        item::Column::Metadata,
                        item::Column::UpdatedAt,
                        item::Column::DeletedAt,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await?;
        Ok(())
    }

    async fn upsert_order_item_event(&self, evt: order_item_event::Model) -> Result<(), DbErr> {
        let am = evt.into_active_model();
        let _ = order_item_event::Entity::insert(am)
            .on_conflict(OnConflict::column(order_item_event::Column::Id).do_nothing().to_owned())
            .exec(&self.db)
            .await;
        Ok(())
    }

    // --- Relay pull ---

    pub async fn pull_and_apply(&self) -> Result<usize, String> {
        info!(
            "SyncEngine: Pulling packets from relay for instance: {}",
            self.instance_id
        );

        let packets = self.relay.pull_packets().await.map_err(|e| {
            error!("Failed to pull from relay: {}", e);
            e.to_string()
        })?;

        if packets.is_empty() {
            info!("SyncEngine: No new packets from relay.");
            return Ok(0);
        }

        info!(
            "SyncEngine: Downloaded {} packets. Processing...",
            packets.len()
        );
        let total = packets.len();
        let mut applied = 0;

        for packet in packets {
            let result = match packet.entity_type.as_str() {
                "product" => self.process_product_packet(&packet).await,
                "location" => self.process_location_packet(&packet).await,
                "user" => self.process_user_packet(&packet).await,
                "order" => self.process_order_packet(&packet).await,
                "document" => self.process_document_packet(&packet).await,
                "file_resource" => self.process_file_resource_packet(&packet).await,
                "attachment" => self.process_attachment_packet(&packet).await,
                "item" => self.process_item_packet(&packet).await,
                "order_item_event" => self.process_order_item_event_packet(&packet).await,
                other => {
                    warn!("Unsupported entity type from relay: {}", other);
                    continue;
                }
            };

            match result {
                Ok(()) => applied += 1,
                Err(e) => error!(
                    "Failed to apply {} packet (entity_id={}): {}",
                    packet.entity_type, packet.entity_id, e
                ),
            }
        }

        info!(
            "SyncEngine: Applied {}/{} packets successfully.",
            applied, total
        );
        Ok(applied)
    }

    async fn process_product_packet(
        &self,
        packet: &crate::models::sync_packet::EncryptedSyncPacket,
    ) -> Result<(), String> {
        let data: product::Model = self
            .security
            .decrypt_packet(packet)
            .map_err(|e| format!("decrypt: {}", e))?;
        self.upsert_product(data)
            .await
            .map_err(|e| format!("upsert: {}", e))
    }

    async fn process_location_packet(
        &self,
        packet: &crate::models::sync_packet::EncryptedSyncPacket,
    ) -> Result<(), String> {
        let data: location::Model = self
            .security
            .decrypt_packet(packet)
            .map_err(|e| format!("decrypt: {}", e))?;
        self.upsert_location(data)
            .await
            .map_err(|e| format!("upsert: {}", e))
    }

    async fn process_user_packet(
        &self,
        packet: &crate::models::sync_packet::EncryptedSyncPacket,
    ) -> Result<(), String> {
        let data: SyncableUser = self
            .security
            .decrypt_packet(packet)
            .map_err(|e| format!("decrypt: {}", e))?;
        self.upsert_user(data)
            .await
            .map_err(|e| format!("upsert: {}", e))
    }

    async fn process_order_packet(&self, packet: &crate::models::sync_packet::EncryptedSyncPacket) -> Result<(), String> {
        let data: SyncableOrder = self.security.decrypt_packet(packet).map_err(|e| format!("decrypt: {}", e))?;
        self.upsert_order(data).await.map_err(|e| format!("upsert: {}", e))
    }

    async fn process_document_packet(&self, packet: &crate::models::sync_packet::EncryptedSyncPacket) -> Result<(), String> {
        let data: SyncableDocument = self.security.decrypt_packet(packet).map_err(|e| format!("decrypt: {}", e))?;
        self.upsert_document(data).await.map_err(|e| format!("upsert: {}", e))
    }

    async fn process_file_resource_packet(&self, packet: &crate::models::sync_packet::EncryptedSyncPacket) -> Result<(), String> {
        let data: SyncableFileResource = self.security.decrypt_packet(packet).map_err(|e| format!("decrypt: {}", e))?;
        self.upsert_file_resource(data).await.map_err(|e| format!("upsert: {}", e))
    }

    async fn process_attachment_packet(&self, packet: &crate::models::sync_packet::EncryptedSyncPacket) -> Result<(), String> {
        let data: attachment::Model = self.security.decrypt_packet(packet).map_err(|e| format!("decrypt: {}", e))?;
        self.upsert_attachment(data).await.map_err(|e| format!("upsert: {}", e))
    }

    async fn process_item_packet(&self, packet: &crate::models::sync_packet::EncryptedSyncPacket) -> Result<(), String> {
        let data: item::Model = self.security.decrypt_packet(packet).map_err(|e| format!("decrypt: {}", e))?;
        self.upsert_item(data).await.map_err(|e| format!("upsert: {}", e))
    }

    async fn process_order_item_event_packet(&self, packet: &crate::models::sync_packet::EncryptedSyncPacket) -> Result<(), String> {
        let data: order_item_event::Model = self.security.decrypt_packet(packet).map_err(|e| format!("decrypt: {}", e))?;
        self.upsert_order_item_event(data).await.map_err(|e| format!("upsert: {}", e))
    }

    // --- Relay push ---

    pub async fn push_entity<T: Serialize>(
        &self,
        target_instance: &str,
        entity_type: &str,
        entity_id: &str,
        payload: &T,
    ) -> Result<(), String> {
        let mut vc = VectorClock::new();
        vc.increment(&self.instance_id);

        let metadata = EntityMetadata {
            entity_id: entity_id.to_string(),
            entity_type: entity_type.to_string(),
            version: 1,
            updated_at: Utc::now(),
            source: "local_server".to_string(),
            source_priority: 80,
            instance_id: self.instance_id.clone(),
            device_id: None,
            vector_clock: vc,
        };

        let encrypted_packet = self
            .security
            .encrypt_packet(&metadata, payload)
            .map_err(|e| {
                error!("Encryption failed: {}", e);
                e.to_string()
            })?;

        self.relay
            .push_packet(target_instance, &encrypted_packet, Some(86400))
            .await
            .map_err(|e| {
                error!("Push to relay failed: {}", e);
                e.to_string()
            })?;

        info!(
            "SyncEngine: Pushed {} [{}] to relay for target '{}'",
            entity_type, entity_id, target_instance
        );
        Ok(())
    }

    // --- Full sync (no merkle, for entity types without checksums) ---

    /// Pull all entities of a given type from a peer (bypasses merkle tree).
    /// Useful for users where entity_checksums isn't maintained.
    pub async fn full_pull_from_peer(&self, peer_url: &str, entity_type: &str) -> anyhow::Result<usize> {
        info!("SyncEngine: Full pull '{}' from {}", entity_type, peer_url);
        let client = MeshClient::new(peer_url);
        let response = client.pull_entities(entity_type, vec![]).await?;
        let count = response.users.len()
            + response.products.len()
            + response.locations.len()
            + response.shipments.len()
            + response.orders.len()
            + response.documents.len()
            + response.file_resources.len()
            + response.attachments.len()
            + response.items.len()
            + response.order_item_events.len();
        self.apply_pull_response(response).await?;
        info!("SyncEngine: Full pull '{}' from {} — {} entities applied", entity_type, peer_url, count);
        Ok(count)
    }

    // --- Mesh sync ---

    pub async fn sync_with_peer(&self, peer_url: &str, entity_type: &str) -> anyhow::Result<()> {
        info!(
            "SyncEngine: Starting active sync with {} for '{}'",
            peer_url, entity_type
        );

        let client = MeshClient::new(peer_url);
        let merkle_svc = MerkleTreeService::new(&self.db);

        // 1. Get root states
        let local_root = merkle_svc
            .get_state(&MerkleRequest {
                entity_type: entity_type.to_string(),
                level: 0,
                bucket: None,
            })
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        let remote_root = client.get_merkle_root(entity_type).await?;

        if local_root.hash == remote_root.hash {
            info!("SyncEngine: Roots match for '{}', no sync needed.", entity_type);
            return Ok(());
        }

        // 2. Compare buckets
        let (buckets_to_pull, buckets_to_push) =
            compare_trees(&local_root.children, &remote_root.children);

        // 3. Drill down into differing buckets to find specific entity IDs
        let mut all_diff_buckets = buckets_to_pull.clone();
        for b in &buckets_to_push {
            if !all_diff_buckets.contains(b) {
                all_diff_buckets.push(b.clone());
            }
        }

        let mut pull_ids = Vec::new();
        let mut push_ids = Vec::new();

        for bucket in &all_diff_buckets {
            let local_bucket = merkle_svc
                .get_state(&MerkleRequest {
                    entity_type: entity_type.to_string(),
                    level: 1,
                    bucket: Some(bucket.clone()),
                })
                .await
                .map_err(|e| anyhow::anyhow!(e))?;

            let remote_children = match client.get_merkle_bucket(entity_type, bucket).await {
                Ok(rb) => rb.children,
                Err(_) => BTreeMap::new(),
            };

            let (diff_pull, diff_push) = compare_trees(&local_bucket.children, &remote_children);
            pull_ids.extend(diff_pull);
            push_ids.extend(diff_push);
        }

        // 4. Pull from remote
        if !pull_ids.is_empty() {
            info!(
                "SyncEngine: Pulling {} '{}' items from {}",
                pull_ids.len(),
                entity_type,
                peer_url
            );
            let response = client.pull_entities(entity_type, pull_ids).await?;
            self.apply_pull_response(response).await?;
        }

        // 5. Push to remote
        if !push_ids.is_empty() {
            info!(
                "SyncEngine: Pushing {} '{}' items to {}",
                push_ids.len(),
                entity_type,
                peer_url
            );
            self.perform_push(&client, entity_type, push_ids).await?;
        }

        info!("SyncEngine: Active sync with {} for '{}' complete.", peer_url, entity_type);
        Ok(())
    }

    async fn apply_pull_response(
        &self,
        resp: crate::handlers::mesh_sync::PullResponse,
    ) -> anyhow::Result<()> {
        // Record checksums for all pulled entities
        let checksum_payload = crate::handlers::mesh_sync::PushPayload {
            products: resp.products.clone(),
            locations: resp.locations.clone(),
            shipments: resp.shipments.clone(),
            users: resp.users.clone(),
            orders: resp.orders.clone(),
            documents: resp.documents.clone(),
            file_resources: resp.file_resources.clone(),
            attachments: resp.attachments.clone(),
            items: resp.items.clone(),
            order_item_events: resp.order_item_events.clone(),
        };
        crate::utils::checksum::record_payload_checksums(&self.db, &checksum_payload, &self.instance_id).await;

        for p in resp.products {
            self.upsert_product(p).await?;
        }
        for l in resp.locations {
            self.upsert_location(l).await?;
        }
        for s in resp.shipments {
            self.upsert_shipment(s).await?;
        }
        for u in resp.users {
            self.upsert_user(u).await?;
        }
        for o in resp.orders {
            self.upsert_order(o).await?;
        }
        for d in resp.documents {
            self.upsert_document(d).await?;
        }
        // File resources must be upserted before attachments (FK constraint)
        for f in resp.file_resources {
            self.upsert_file_resource(f).await?;
        }
        for a in resp.attachments {
            self.upsert_attachment(a).await?;
        }
        for it in resp.items {
            self.upsert_item(it).await?;
        }
        for evt in resp.order_item_events {
            self.upsert_order_item_event(evt).await?;
        }
        Ok(())
    }

    async fn perform_push(
        &self,
        client: &MeshClient,
        entity_type: &str,
        ids: Vec<String>,
    ) -> anyhow::Result<()> {
        use sea_orm::{ColumnTrait, QueryFilter};

        let mut products = vec![];
        let mut locations = vec![];
        let mut shipments = vec![];
        let mut users = vec![];
        let mut orders = vec![];
        let mut documents = vec![];
        let mut file_resources = vec![];
        let mut attachments = vec![];
        let mut items = vec![];
        let mut order_item_events_vec = vec![];

        match entity_type {
            "user" => {
                let parsed_uuids: Vec<uuid::Uuid> = ids.iter().filter_map(|s| s.parse().ok()).collect();
                if !parsed_uuids.is_empty() {
                    let user_models = user::Entity::find()
                        .filter(user::Column::Id.is_in(parsed_uuids))
                        .filter(user::Column::Email.ne("admin@setup.local"))
                        .all(&self.db)
                        .await?;
                    users = user_models.into_iter().map(SyncableUser::from).collect();
                }
            }
            "order" => {
                let parsed_uuids: Vec<uuid::Uuid> = ids.iter().filter_map(|s| s.parse().ok()).collect();
                if !parsed_uuids.is_empty() {
                    let models = order::Entity::find()
                        .filter(order::Column::Id.is_in(parsed_uuids))
                        .all(&self.db)
                        .await?;
                    orders = models.into_iter().map(SyncableOrder::from).collect();
                }
            }
            "document" => {
                let parsed_uuids: Vec<uuid::Uuid> = ids.iter().filter_map(|s| s.parse().ok()).collect();
                if !parsed_uuids.is_empty() {
                    let models = document::Entity::find()
                        .filter(document::Column::Id.is_in(parsed_uuids))
                        .all(&self.db)
                        .await?;
                    documents = models.into_iter().map(SyncableDocument::from).collect();
                }
            }
            "file_resource" => {
                let parsed_uuids: Vec<uuid::Uuid> = ids.iter().filter_map(|s| s.parse().ok()).collect();
                if !parsed_uuids.is_empty() {
                    let models = file_resource::Entity::find()
                        .filter(file_resource::Column::Id.is_in(parsed_uuids))
                        .all(&self.db)
                        .await?;
                    file_resources = models.into_iter().map(SyncableFileResource::from).collect();
                }
            }
            "attachment" => {
                let parsed_uuids: Vec<uuid::Uuid> = ids.iter().filter_map(|s| s.parse().ok()).collect();
                if !parsed_uuids.is_empty() {
                    attachments = attachment::Entity::find()
                        .filter(attachment::Column::Id.is_in(parsed_uuids))
                        .all(&self.db)
                        .await?;
                }
            }
            "item" => {
                let parsed_uuids: Vec<uuid::Uuid> = ids.iter().filter_map(|s| s.parse().ok()).collect();
                if !parsed_uuids.is_empty() {
                    items = item::Entity::find()
                        .filter(item::Column::Id.is_in(parsed_uuids))
                        .all(&self.db)
                        .await?;
                }
            }
            "order_item_event" => {
                let parsed_uuids: Vec<uuid::Uuid> = ids.iter().filter_map(|s| s.parse().ok()).collect();
                if !parsed_uuids.is_empty() {
                    order_item_events_vec = order_item_event::Entity::find()
                        .filter(order_item_event::Column::Id.is_in(parsed_uuids))
                        .all(&self.db)
                        .await?;
                }
            }
            "product" => {
                let parsed_uuids: Vec<uuid::Uuid> = ids.iter().filter_map(|s| s.parse().ok()).collect();
                if !parsed_uuids.is_empty() {
                    products = product::Entity::find()
                        .filter(product::Column::Id.is_in(parsed_uuids))
                        .all(&self.db).await?;
                }
            }
            "location" => {
                let parsed_uuids: Vec<uuid::Uuid> = ids.iter().filter_map(|s| s.parse().ok()).collect();
                if !parsed_uuids.is_empty() {
                    locations = location::Entity::find()
                        .filter(location::Column::Id.is_in(parsed_uuids))
                        .all(&self.db).await?;
                }
            }
            "shipment" => {
                let parsed_uuids: Vec<uuid::Uuid> = ids.iter().filter_map(|s| s.parse().ok()).collect();
                if !parsed_uuids.is_empty() {
                    shipments = stock_picking_delivery::Entity::find()
                        .filter(stock_picking_delivery::Column::Id.is_in(parsed_uuids))
                        .all(&self.db).await?;
                }
            }
            _ => {}
        }

        client
            .push_entities(products, locations, shipments, users, orders, documents, file_resources, attachments, items, order_item_events_vec)
            .await?;
        Ok(())
    }
}
