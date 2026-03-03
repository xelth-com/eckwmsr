use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use sha2::{Digest, Sha256};
use std::io::Cursor;
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn};
use uuid::Uuid;

use crate::models::file_resource;

/// Max size for storing content inline in DB as avatar (50KB).
const MAX_AVATAR_SIZE: usize = 50 * 1024;

/// Compute a deterministic UUID from file bytes using MurmurHash3 x64_128 (seed=0).
/// Matches the Kotlin/Android ContentHash.uuidFromBytes() implementation exactly.
///
/// The murmur3 crate returns u128 where the upper 64 bits = h1, lower 64 bits = h2.
/// We format as UUID: h1 → first 8 bytes (big-endian), h2 → last 8 bytes (big-endian).
pub fn content_hash_uuid(data: &[u8]) -> Uuid {
    let hash = murmur3::murmur3_x64_128(&mut Cursor::new(data), 0)
        .expect("murmur3 hash should not fail on in-memory data");
    // murmur3 crate packs u128 as: lower 64 bits = h1, upper 64 bits = h2
    let h1 = hash as u64;
    let h2 = (hash >> 64) as u64;
    // Construct 16 bytes: h1 big-endian || h2 big-endian (matches Kotlin)
    let mut bytes = [0u8; 16];
    bytes[0..8].copy_from_slice(&h1.to_be_bytes());
    bytes[8..16].copy_from_slice(&h2.to_be_bytes());
    Uuid::from_bytes(bytes)
}

/// FileStoreService handles persistent file storage with CAS deduplication.
/// Mirrors Go's `filestore.Service` from `internal/services/filestore/service.go`.
pub struct FileStoreService {
    base_dir: String,
}

impl FileStoreService {
    pub fn new(base_dir: &str) -> Self {
        Self {
            base_dir: base_dir.to_string(),
        }
    }

    /// Saves a file using CAS logic (deduplication by content hash).
    /// If `claimed_id` is provided, verifies it matches the computed Murmur3 UUID.
    /// If a file with the same ID already exists, returns the existing record (idempotent).
    /// `explicit_avatar`: optional client-generated thumbnail (Smart Crop 224x224).
    pub async fn save_file(
        &self,
        db: &DatabaseConnection,
        content: &[u8],
        filename: &str,
        mime_type: &str,
        device_id: &str,
        context: &str,
        explicit_avatar: Option<Vec<u8>>,
        claimed_id: Option<&str>,
    ) -> Result<file_resource::Model, String> {
        // 1. Compute deterministic CAS UUID (MurmurHash3 x64_128)
        let cas_uuid = content_hash_uuid(content);

        // 2. Verify claimed_id if provided
        if let Some(claimed) = claimed_id {
            if !claimed.is_empty() {
                if let Ok(parsed) = Uuid::parse_str(claimed) {
                    if parsed != cas_uuid {
                        warn!(
                            "FileStore: CAS mismatch! claimed={} computed={} file={}",
                            claimed, cas_uuid, filename
                        );
                        return Err(format!(
                            "CAS verification failed: claimed {} != computed {}",
                            claimed, cas_uuid
                        ));
                    }
                }
                // Non-UUID claimed_id (legacy) — skip verification
            }
        }

        // 3. Check for duplicate (idempotent — same content = same UUID)
        if let Ok(Some(existing)) = file_resource::Entity::find_by_id(cas_uuid)
            .one(db)
            .await
        {
            info!(
                "FileStore: Deduplicated upload {} -> existing {}",
                filename, existing.id
            );
            return Ok(existing);
        }

        // Also check by SHA-256 hash for backward compat with old uploads
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash_str = hex::encode(hasher.finalize());

        if let Ok(Some(existing)) = file_resource::Entity::find()
            .filter(file_resource::Column::Hash.eq(&hash_str))
            .one(db)
            .await
        {
            info!(
                "FileStore: Deduplicated (SHA256) upload {} -> existing {}",
                filename, existing.id
            );
            return Ok(existing);
        }

        // 3. Determine storage path (CAS: aa/bb/hash.ext)
        let sub_dir1 = &hash_str[0..2];
        let sub_dir2 = &hash_str[2..4];

        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|os| os.to_str())
            .map(|s| format!(".{}", s))
            .unwrap_or_default();

        let rel_path = format!(
            "data/filestore/{}/{}/{}{}",
            sub_dir1, sub_dir2, hash_str, ext
        );
        let abs_path = PathBuf::from(&self.base_dir).join(&rel_path);

        if let Some(parent) = abs_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // 4. Write to disk
        fs::write(&abs_path, content)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;

        // 5. Avatar: prefer client thumbnail, fallback to inline if small
        let avatar_data = if let Some(avatar) = explicit_avatar.filter(|a| !a.is_empty()) {
            info!("Using client avatar ({} bytes)", avatar.len());
            Some(avatar)
        } else if content.len() <= MAX_AVATAR_SIZE {
            Some(content.to_vec())
        } else {
            None
        };

        // 6. Create DB record
        let now = Utc::now();
        let new_file = file_resource::ActiveModel {
            id: Set(cas_uuid),
            hash: Set(hash_str),
            original_name: Set(filename.to_string()),
            mime_type: Set(mime_type.to_string()),
            size_bytes: Set(content.len() as i64),
            width: Set(0),
            height: Set(0),
            avatar_data: Set(avatar_data),
            storage_path: Set(rel_path),
            created_by_device: Set(device_id.to_string()),
            context: Set(context.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };

        let saved = new_file
            .insert(db)
            .await
            .map_err(|e| format!("Failed to save file record: {}", e))?;

        info!(
            "FileStore: Saved {} ({} bytes) as {}",
            filename,
            content.len(),
            saved.id
        );
        Ok(saved)
    }

    /// Reads file content — prefers inline avatar (fast), falls back to disk (slower).
    pub async fn get_file_content(
        &self,
        file: &file_resource::Model,
    ) -> Result<Vec<u8>, String> {
        if let Some(ref avatar) = file.avatar_data {
            if !avatar.is_empty() {
                return Ok(avatar.clone());
            }
        }
        let abs_path = PathBuf::from(&self.base_dir).join(&file.storage_path);
        fs::read(&abs_path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_uuid_vectors() {
        // Cross-platform reference vectors (must match Kotlin ContentHash.uuidFromBytes)
        assert_eq!(content_hash_uuid(b"test").to_string(), "ac7d28cc-74bd-e19d-9a12-8231f9bd4d82");
        assert_eq!(content_hash_uuid(b"hello").to_string(), "cbd8a7b3-41bd-9b02-5b1e-906a48ae1d19");
        assert_eq!(content_hash_uuid(b"").to_string(), "00000000-0000-0000-0000-000000000000");
        // Determinism
        assert_eq!(content_hash_uuid(b"test").to_string(), content_hash_uuid(b"test").to_string());
    }
}
