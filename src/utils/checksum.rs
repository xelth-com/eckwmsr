use serde::Serialize;
use sha2::{Digest, Sha256};
use serde_json::Value;
use std::collections::BTreeMap;

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
