use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::Serialize;
use sha2::{Digest, Sha256};
use serde_json::Value;
use std::collections::BTreeMap;
use tracing::warn;

use crate::handlers::mesh_sync::PushPayload;
use crate::models::checksum;

/// ChecksumCalculator generates deterministic SHA-256 hashes for entities.
/// Mirrors Go's `ChecksumCalculator` from `internal/sync/checksum.go`.
pub struct ChecksumCalculator {
    pub instance_id: String,
}

impl ChecksumCalculator {
    pub fn new(instance_id: String) -> Self {
        Self { instance_id }
    }

    /// Computes a deterministic checksum, ignoring timestamp fields.
    /// Uses sorted keys (BTreeMap) and canonical string format matching Go.
    pub fn compute_checksum<T: Serialize>(&self, entity: &T) -> Result<String, String> {
        let val = serde_json::to_value(entity).map_err(|e| e.to_string())?;

        let mut map: BTreeMap<String, Value> = match val {
            Value::Object(m) => m.into_iter().collect(),
            _ => return Err("Entity must serialize to an object".to_string()),
        };

        // Remove ignored fields — all casing variants (matches Go exactly)
        let ignored_keys = [
            "created_at",
            "updated_at",
            "last_synced_at",
            "CreatedAt",
            "UpdatedAt",
            "LastSyncedAt",
            "createdAt",
            "updatedAt",
            "lastSyncedAt",
        ];

        for k in &ignored_keys {
            map.remove(*k);
        }

        // Build canonical string (BTreeMap is already sorted by key)
        let mut canonical = String::new();
        for (k, v) in &map {
            if v.is_null() {
                canonical.push_str(&format!("{}:null;", k));
            } else if let Some(s) = v.as_str() {
                // Normalize RFC3339 timestamps to UTC (matching Go behavior)
                if let Ok(t) = chrono::DateTime::parse_from_rfc3339(s) {
                    canonical.push_str(&format!(
                        "{}:{};",
                        k,
                        t.with_timezone(&chrono::Utc).to_rfc3339()
                    ));
                    continue;
                }
                canonical.push_str(&format!("{}:{};", k, s));
            } else {
                canonical.push_str(&format!("{}:{};", k, v));
            }
        }

        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let result = hasher.finalize();

        Ok(hex::encode(result))
    }
}

/// Upsert a single entity checksum row (select + insert/update).
async fn upsert_checksum(
    db: &DatabaseConnection,
    entity_type: &str,
    entity_id: &str,
    hash: &str,
    source_instance: &str,
) {
    let now = Utc::now();
    let existing = checksum::Entity::find()
        .filter(checksum::Column::EntityType.eq(entity_type))
        .filter(checksum::Column::EntityId.eq(entity_id))
        .one(db)
        .await;

    match existing {
        Ok(Some(row)) => {
            if row.full_hash == hash {
                return; // unchanged
            }
            let mut am: checksum::ActiveModel = row.into();
            am.content_hash = Set(hash.to_string());
            am.full_hash = Set(hash.to_string());
            am.last_updated = Set(now);
            am.updated_at = Set(now);
            am.source_instance = Set(source_instance.to_string());
            if let Err(e) = am.update(db).await {
                warn!("checksum update failed for {} {}: {}", entity_type, entity_id, e);
            }
        }
        Ok(None) => {
            let am = checksum::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                entity_type: Set(entity_type.to_string()),
                entity_id: Set(entity_id.to_string()),
                content_hash: Set(hash.to_string()),
                children_hash: Set(None),
                full_hash: Set(hash.to_string()),
                child_count: Set(0),
                last_updated: Set(now),
                source_instance: Set(source_instance.to_string()),
                source_device: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
            };
            if let Err(e) = am.insert(db).await {
                warn!("checksum insert failed for {} {}: {}", entity_type, entity_id, e);
            }
        }
        Err(e) => {
            warn!("checksum lookup failed for {} {}: {}", entity_type, entity_id, e);
        }
    }
}

/// Record checksums for all entities in a PushPayload.
/// Called after local mutations and incoming syncs to keep the Merkle tree up to date.
pub async fn record_payload_checksums(
    db: &DatabaseConnection,
    payload: &PushPayload,
    instance_id: &str,
) {
    let calc = ChecksumCalculator::new(instance_id.to_string());

    for p in &payload.products {
        if let Ok(hash) = calc.compute_checksum(p) {
            upsert_checksum(db, "product", &p.id.to_string(), &hash, instance_id).await;
        }
    }
    for l in &payload.locations {
        if let Ok(hash) = calc.compute_checksum(l) {
            upsert_checksum(db, "location", &l.id.to_string(), &hash, instance_id).await;
        }
    }
    for s in &payload.shipments {
        if let Ok(hash) = calc.compute_checksum(s) {
            upsert_checksum(db, "shipment", &s.id.to_string(), &hash, instance_id).await;
        }
    }
    for u in &payload.users {
        if let Ok(hash) = calc.compute_checksum(u) {
            upsert_checksum(db, "user", &u.id.to_string(), &hash, instance_id).await;
        }
    }
    for o in &payload.orders {
        if let Ok(hash) = calc.compute_checksum(o) {
            upsert_checksum(db, "order", &o.id.to_string(), &hash, instance_id).await;
        }
    }
    for d in &payload.documents {
        if let Ok(hash) = calc.compute_checksum(d) {
            upsert_checksum(db, "document", &d.id.to_string(), &hash, instance_id).await;
        }
    }
    for f in &payload.file_resources {
        if let Ok(hash) = calc.compute_checksum(f) {
            upsert_checksum(db, "file_resource", &f.id.to_string(), &hash, instance_id).await;
        }
    }
    for a in &payload.attachments {
        if let Ok(hash) = calc.compute_checksum(a) {
            upsert_checksum(db, "attachment", &a.id.to_string(), &hash, instance_id).await;
        }
    }
    for it in &payload.items {
        if let Ok(hash) = calc.compute_checksum(it) {
            upsert_checksum(db, "item", &it.id.to_string(), &hash, instance_id).await;
        }
    }
    for evt in &payload.order_item_events {
        if let Ok(hash) = calc.compute_checksum(evt) {
            upsert_checksum(db, "order_item_event", &evt.id.to_string(), &hash, instance_id).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestEntity {
        id: i64,
        name: String,
        #[serde(rename = "createdAt")]
        created_at: String,
        #[serde(rename = "updatedAt")]
        updated_at: String,
    }

    #[test]
    fn test_checksum_ignores_timestamps() {
        let calc = ChecksumCalculator::new("test".to_string());

        let e1 = TestEntity {
            id: 1,
            name: "test".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };
        let e2 = TestEntity {
            id: 1,
            name: "test".to_string(),
            created_at: "2026-06-15T12:00:00Z".to_string(),
            updated_at: "2026-06-15T12:00:00Z".to_string(),
        };

        let h1 = calc.compute_checksum(&e1).unwrap();
        let h2 = calc.compute_checksum(&e2).unwrap();
        assert_eq!(h1, h2, "Checksums should match when only timestamps differ");
    }

    #[test]
    fn test_checksum_detects_content_change() {
        let calc = ChecksumCalculator::new("test".to_string());

        let e1 = TestEntity {
            id: 1,
            name: "alpha".to_string(),
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };
        let e2 = TestEntity {
            id: 1,
            name: "beta".to_string(),
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };

        let h1 = calc.compute_checksum(&e1).unwrap();
        let h2 = calc.compute_checksum(&e2).unwrap();
        assert_ne!(h1, h2, "Checksums should differ when content changes");
    }
}
