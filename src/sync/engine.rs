use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{sea_query::OnConflict, IntoActiveModel};
use serde::Serialize;
use tracing::{error, info, warn};

use crate::models::sync_packet::EntityMetadata;
use crate::models::{location, product};
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

    /// Returns a clone of the relay client for use in other services
    pub fn relay_client(&self) -> RelayClient {
        self.relay.clone()
    }

    /// Pulls pending packets from the blind relay, decrypts them, and upserts into local DB.
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
                "product" => self.apply_product(&packet).await,
                "location" => self.apply_location(&packet).await,
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

    async fn apply_product(
        &self,
        packet: &crate::models::sync_packet::EncryptedSyncPacket,
    ) -> Result<(), String> {
        let data: product::Model = self
            .security
            .decrypt_packet(packet)
            .map_err(|e| format!("decrypt: {}", e))?;

        let am = data.into_active_model();
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
            .await
            .map_err(|e| format!("upsert: {}", e))?;

        Ok(())
    }

    /// Pushes a local entity to the blind relay for a specific target instance.
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

    async fn apply_location(
        &self,
        packet: &crate::models::sync_packet::EncryptedSyncPacket,
    ) -> Result<(), String> {
        let data: location::Model = self
            .security
            .decrypt_packet(packet)
            .map_err(|e| format!("decrypt: {}", e))?;

        let am = data.into_active_model();
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
            .await
            .map_err(|e| format!("upsert: {}", e))?;

        Ok(())
    }
}
