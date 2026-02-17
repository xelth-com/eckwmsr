use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};

use crate::models::checksum;

/// Represents a node in the Merkle Tree
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MerkleNode {
    pub level: u8,
    pub key: String,
    pub hash: String,
    pub children: BTreeMap<String, String>, // Key -> Hash
}

impl MerkleNode {
    pub fn new(level: u8, key: String) -> Self {
        Self {
            level,
            key,
            hash: String::new(),
            children: BTreeMap::new(),
        }
    }
}

/// Request payload for Merkle comparison
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MerkleRequest {
    pub entity_type: String,
    pub level: u8,
    pub bucket: Option<String>,
}

/// Service to build Merkle Trees from entity_checksums table
pub struct MerkleTreeService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> MerkleTreeService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get tree state at given level
    pub async fn get_state(&self, req: &MerkleRequest) -> Result<MerkleNode, String> {
        match req.level {
            0 => self.get_root(&req.entity_type).await,
            1 if req.bucket.is_some() => {
                self.get_bucket(&req.entity_type, req.bucket.as_ref().unwrap())
                    .await
            }
            _ => Err("Invalid merkle request: level must be 0 or 1 with bucket".to_string()),
        }
    }

    /// Level 0: root node with bucket hashes as children
    async fn get_root(&self, entity_type: &str) -> Result<MerkleNode, String> {
        let checksums = checksum::Entity::find()
            .filter(checksum::Column::EntityType.eq(entity_type))
            .all(self.db)
            .await
            .map_err(|e| e.to_string())?;

        let mut bucket_items: HashMap<String, BTreeMap<String, String>> = HashMap::new();

        for cs in checksums {
            let b_key = get_bucket_index(&cs.entity_id);
            bucket_items
                .entry(b_key)
                .or_default()
                .insert(cs.entity_id, cs.full_hash);
        }

        let mut buckets: BTreeMap<String, String> = BTreeMap::new();
        for (b_key, items) in &bucket_items {
            buckets.insert(b_key.clone(), compute_bucket_hash(items));
        }

        let root_hash = compute_root_hash(&buckets);

        Ok(MerkleNode {
            level: 0,
            key: "root".to_string(),
            hash: root_hash,
            children: buckets,
        })
    }

    /// Level 1: single bucket with entity_id -> hash as children
    async fn get_bucket(&self, entity_type: &str, bucket: &str) -> Result<MerkleNode, String> {
        let checksums = checksum::Entity::find()
            .filter(checksum::Column::EntityType.eq(entity_type))
            .all(self.db)
            .await
            .map_err(|e| e.to_string())?;

        let mut items: BTreeMap<String, String> = BTreeMap::new();
        for cs in checksums {
            if get_bucket_index(&cs.entity_id) == *bucket {
                items.insert(cs.entity_id, cs.full_hash);
            }
        }

        let hash = compute_bucket_hash(&items);

        Ok(MerkleNode {
            level: 1,
            key: bucket.to_string(),
            hash,
            children: items,
        })
    }
}

/// Compute hash of a bucket of items (sorted by key for determinism)
pub fn compute_bucket_hash(items: &BTreeMap<String, String>) -> String {
    let mut hasher = Sha256::new();
    for (id, hash) in items {
        hasher.update(id.as_bytes());
        hasher.update(b":");
        hasher.update(hash.as_bytes());
        hasher.update(b";");
    }
    hex::encode(hasher.finalize())
}

/// Compute root hash from bucket hashes
pub fn compute_root_hash(buckets: &BTreeMap<String, String>) -> String {
    let mut hasher = Sha256::new();
    for (bucket_key, hash) in buckets {
        hasher.update(bucket_key.as_bytes());
        hasher.update(b":");
        hasher.update(hash.as_bytes());
        hasher.update(b";");
    }
    hex::encode(hasher.finalize())
}

/// Compare two sets of bucket/entity hashes.
/// Returns (need_from_remote, need_to_push) keys.
pub fn compare_trees(
    local: &BTreeMap<String, String>,
    remote: &BTreeMap<String, String>,
) -> (Vec<String>, Vec<String>) {
    let mut need_from_remote = Vec::new();
    let mut need_to_push = Vec::new();

    for (r_key, r_hash) in remote {
        match local.get(r_key) {
            Some(l_hash) if l_hash == r_hash => {}
            Some(_) => {
                // Both have it but different hash
                need_from_remote.push(r_key.clone());
                need_to_push.push(r_key.clone());
            }
            None => need_from_remote.push(r_key.clone()),
        }
    }

    for l_key in local.keys() {
        if !remote.contains_key(l_key) {
            need_to_push.push(l_key.clone());
        }
    }

    (need_from_remote, need_to_push)
}

/// Simple bucketing by first character of entity_id
pub fn get_bucket_index(entity_id: &str) -> String {
    entity_id
        .chars()
        .next()
        .unwrap_or('_')
        .to_lowercase()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_hash_deterministic() {
        let mut items = BTreeMap::new();
        items.insert("a".into(), "hash_a".into());
        items.insert("b".into(), "hash_b".into());
        let h1 = compute_bucket_hash(&items);
        let h2 = compute_bucket_hash(&items);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_bucket_hash_differs() {
        let mut a = BTreeMap::new();
        a.insert("x".into(), "1".into());
        let mut b = BTreeMap::new();
        b.insert("x".into(), "2".into());
        assert_ne!(compute_bucket_hash(&a), compute_bucket_hash(&b));
    }

    #[test]
    fn test_compare_trees() {
        let mut local = BTreeMap::new();
        local.insert("a".into(), "hash1".into());
        local.insert("b".into(), "hash2".into());
        local.insert("c".into(), "hash3".into());

        let mut remote = BTreeMap::new();
        remote.insert("a".into(), "hash1".into());
        remote.insert("b".into(), "changed".into());
        remote.insert("d".into(), "hash4".into());

        let (need_from_remote, need_to_push) = compare_trees(&local, &remote);
        assert!(need_from_remote.contains(&"b".to_string()));
        assert!(need_from_remote.contains(&"d".to_string()));
        assert!(need_to_push.contains(&"c".to_string()));
        assert!(need_to_push.contains(&"b".to_string())); // both have "b" but different
        assert!(!need_to_push.contains(&"a".to_string()));
    }

    #[test]
    fn test_get_bucket_index() {
        assert_eq!(get_bucket_index("Product-123"), "p");
        assert_eq!(get_bucket_index("abc"), "a");
        assert_eq!(get_bucket_index(""), "_");
    }
}
