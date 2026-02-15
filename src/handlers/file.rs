use axum::{
    body::Bytes,
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::Serialize;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    db::AppState,
    models::{attachment, file_resource},
    utils::smart_code::generate_smart_item,
};

#[derive(Serialize)]
pub struct AttachmentResult {
    pub id: String,
    pub file_id: String,
    pub mime_type: String,
    pub is_main: bool,
    pub created_at: chrono::DateTime<Utc>,
}

/// POST /api/upload/image — multipart image upload with CAS dedup and auto-linking.
/// Mirrors Go's `handleImageUpload` from `internal/handlers/upload.go`.
pub async fn handle_image_upload(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut image_data: Option<Bytes> = None;
    let mut avatar_data: Option<Vec<u8>> = None;
    let mut filename = String::new();
    let mut mime_type = String::from("image/jpeg");

    let mut device_id = String::new();
    let mut scan_mode = String::new();
    let mut barcode_data = String::new();
    let mut order_id = String::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "image" => {
                filename = field
                    .file_name()
                    .unwrap_or("upload.jpg")
                    .to_string();
                if let Some(ct) = field.content_type() {
                    mime_type = ct.to_string();
                }
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
                image_data = Some(data);
            }
            "avatar" => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
                avatar_data = Some(data.to_vec());
            }
            "deviceId" => device_id = field.text().await.unwrap_or_default(),
            "scanMode" => scan_mode = field.text().await.unwrap_or_default(),
            "barcodeData" => barcode_data = field.text().await.unwrap_or_default(),
            "orderId" => order_id = field.text().await.unwrap_or_default(),
            _ => {}
        }
    }

    let content =
        image_data.ok_or((StatusCode::BAD_REQUEST, "Missing 'image' field".to_string()))?;

    let context_str = if order_id.is_empty() {
        scan_mode.clone()
    } else {
        format!("{}:{}", scan_mode, order_id)
    };

    let file_res = state
        .file_store
        .save_file(
            &state.db,
            &content,
            &filename,
            &mime_type,
            &device_id,
            &context_str,
            avatar_data,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // --- AUTO-LINKING LOGIC (Smart Bind) ---
    let barcode_data = barcode_data.trim();
    if !barcode_data.is_empty() {
        let (res_model, res_id) = resolve_barcode_to_entity(barcode_data);

        if !res_model.is_empty() {
            let new_attachment = attachment::ActiveModel {
                id: Set(Uuid::new_v4()),
                file_resource_id: Set(file_res.id),
                res_model: Set(res_model.to_string()),
                res_id: Set(res_id.clone()),
                is_main: Set(true),
                tags: Set(Some(scan_mode)),
                comment: Set(None),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
                deleted_at: Set(None),
            };

            if let Err(e) = new_attachment.insert(&state.db).await {
                warn!("Failed to auto-link attachment: {}", e);
            } else {
                info!("Linked file {} to {}:{}", file_res.id, res_model, res_id);
            }
        }
    }

    Ok(Json(serde_json::json!({
        "status": "uploaded",
        "id": file_res.id,
        "hash": file_res.hash,
        "message": "Image stored successfully (CAS)"
    })))
}

/// GET /api/files/:id — serves file content (avatar from DB or original from disk).
/// Mirrors Go's `serveFile` from `internal/handlers/upload.go`.
pub async fn serve_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let file_res = file_resource::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    let content = state
        .file_store
        .get_file_content(&file_res)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    let source = if file_res
        .avatar_data
        .as_ref()
        .map_or(false, |a| !a.is_empty())
    {
        "db-avatar"
    } else {
        "disk-cas"
    };

    Ok((
        [
            (header::CONTENT_TYPE, file_res.mime_type),
            (header::CACHE_CONTROL, "public, max-age=31536000".to_string()),
            (
                header::HeaderName::from_static("x-content-source"),
                source.to_string(),
            ),
        ],
        Bytes::from(content),
    ))
}

/// GET /api/attachments/:model/:id — lists file attachments for a business entity.
/// Mirrors Go's `listEntityAttachments` from `internal/handlers/upload.go`.
pub async fn list_entity_attachments(
    State(state): State<Arc<AppState>>,
    Path((model, id)): Path<(String, String)>,
) -> Result<Json<Vec<AttachmentResult>>, (StatusCode, String)> {
    // Smart resolution: auto-convert EAN to smart code for product lookups
    let mut lookup_ids = vec![id.clone()];
    if model == "product" && is_numeric_ean(&id) {
        lookup_ids.push(generate_smart_item(String::new(), id.clone()));
    }

    let attachments = attachment::Entity::find()
        .filter(
            Condition::all()
                .add(attachment::Column::ResModel.eq(&model))
                .add(attachment::Column::ResId.is_in(lookup_ids))
                .add(attachment::Column::DeletedAt.is_null()),
        )
        .find_also_related(file_resource::Entity)
        .order_by_desc(attachment::Column::IsMain)
        .order_by_desc(attachment::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let results = attachments
        .into_iter()
        .filter_map(|(att, file_opt)| {
            file_opt.map(|file| AttachmentResult {
                id: att.id.to_string(),
                file_id: file.id.to_string(),
                mime_type: file.mime_type,
                is_main: att.is_main,
                created_at: att.created_at,
            })
        })
        .collect();

    Ok(Json(results))
}

/// Resolves a barcode string to (res_model, res_id) for auto-linking.
fn resolve_barcode_to_entity(barcode: &str) -> (&'static str, String) {
    if barcode.starts_with('i') {
        ("product", barcode.to_string())
    } else if barcode.starts_with('p') || barcode.starts_with("LOC") {
        ("location", barcode.to_string())
    } else if barcode.starts_with('b') {
        ("package", barcode.to_string())
    } else if let Some(ean) = barcode.strip_prefix("ITEM:") {
        if is_numeric_ean(ean) {
            let smart_code = generate_smart_item(String::new(), ean.to_string());
            info!("ITEM:{} -> Smart Code: {}", ean, smart_code);
            ("product", smart_code)
        } else {
            warn!("ITEM:{} is not a numeric EAN, stored as generic", ean);
            ("generic", barcode.to_string())
        }
    } else if is_numeric_ean(barcode) {
        let smart_code = generate_smart_item(String::new(), barcode.to_string());
        info!("EAN:{} -> Smart Code: {}", barcode, smart_code);
        ("product", smart_code)
    } else {
        ("generic", barcode.to_string())
    }
}

fn is_numeric_ean(s: &str) -> bool {
    let len = s.len();
    (8..=14).contains(&len) && s.chars().all(|c| c.is_ascii_digit())
}
