use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};

/// Represents a node in the Merkle Tree
#[derive(Debug, Clone)]
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

/// Compare local tree with remote tree response.
/// Returns (need_from_remote, need_to_push) bucket keys.
pub fn compare_trees(
    local_buckets: &BTreeMap<String, String>,
    remote_buckets: &HashMap<String, String>,
) -> (Vec<String>, Vec<String>) {
    let mut need_from_remote = Vec::new();
    let mut need_to_push = Vec::new();

    // What we need from remote (mismatched or missing locally)
    for (r_key, r_hash) in remote_buckets {
        match local_buckets.get(r_key) {
            Some(l_hash) if l_hash == r_hash => {} // identical
            _ => need_from_remote.push(r_key.clone()),
        }
    }

    // What remote is missing (we have but they don't)
    for l_key in local_buckets.keys() {
        if !remote_buckets.contains_key(l_key) {
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

        let mut remote = HashMap::new();
        remote.insert("a".into(), "hash1".into()); // same
        remote.insert("b".into(), "changed".into()); // different
        remote.insert("d".into(), "hash4".into()); // remote only

        let (need_from_remote, need_to_push) = compare_trees(&local, &remote);
        assert!(need_from_remote.contains(&"b".to_string()));
        assert!(need_from_remote.contains(&"d".to_string()));
        assert!(need_to_push.contains(&"c".to_string()));
        assert!(!need_to_push.contains(&"a".to_string()));
    }

    #[test]
    fn test_get_bucket_index() {
        assert_eq!(get_bucket_index("Product-123"), "p");
        assert_eq!(get_bucket_index("abc"), "a");
        assert_eq!(get_bucket_index(""), "_");
    }
}
