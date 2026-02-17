use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{sea_query::OnConflict, IntoActiveModel};
use serde::Serialize;
use tracing::{error, info, warn};

use std::collections::BTreeMap;

use crate::models::sync_packet::EntityMetadata;
use crate::models::{location, product, stock_picking_delivery};
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
        for p in resp.products {
            self.upsert_product(p).await?;
        }
        for l in resp.locations {
            self.upsert_location(l).await?;
        }
        for s in resp.shipments {
            self.upsert_shipment(s).await?;
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

        let parsed_ids: Vec<i64> = ids.iter().filter_map(|s| s.parse().ok()).collect();
        if parsed_ids.is_empty() {
            return Ok(());
        }

        let mut products = vec![];
        let mut locations = vec![];
        let mut shipments = vec![];

        match entity_type {
            "product" => {
                products = product::Entity::find()
                    .filter(product::Column::Id.is_in(parsed_ids))
                    .all(&self.db)
                    .await?;
            }
            "location" => {
                locations = location::Entity::find()
                    .filter(location::Column::Id.is_in(parsed_ids))
                    .all(&self.db)
                    .await?;
            }
            "shipment" => {
                shipments = stock_picking_delivery::Entity::find()
                    .filter(stock_picking_delivery::Column::Id.is_in(parsed_ids))
                    .all(&self.db)
                    .await?;
            }
            _ => {}
        }

        client.push_entities(products, locations, shipments).await?;
        Ok(())
    }
}
