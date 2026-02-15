use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use uuid::Uuid;

use crate::models::file_resource;

/// Max size for storing content inline in DB as avatar (50KB).
const MAX_AVATAR_SIZE: usize = 50 * 1024;

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

    /// Saves a file using CAS logic (deduplication by SHA-256 hash).
    /// If a file with the same hash already exists, returns the existing record.
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
    ) -> Result<file_resource::Model, String> {
        // 1. Calculate SHA256 hash
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash_str = hex::encode(hasher.finalize());

        // 2. Check for duplicate (CAS dedup)
        if let Ok(Some(existing)) = file_resource::Entity::find()
            .filter(file_resource::Column::Hash.eq(&hash_str))
            .one(db)
            .await
        {
            info!(
                "FileStore: Deduplicated upload {} -> existing {}",
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
            id: Set(Uuid::new_v4()),
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
