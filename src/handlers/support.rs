use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use base64::Engine;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    db::AppState,
    models::{attachment, document},
};

const SCRAPER_BASE: &str = "http://127.0.0.1:3211";

// ── Read handlers ─────────────────────────────────────────────────────────────

/// A compact summary of a unique support ticket derived from its imported threads.
#[derive(Serialize)]
pub struct TicketSummary {
    pub ticket_id: String,
    pub subject: String,
    pub status: String,
    pub customer: String,
    pub thread_count: usize,
    pub latest_update: String,
}

/// GET /api/support/tickets
///
/// Lists all unique support tickets that have been imported into the `documents`
/// table (type = "support_thread"). Tickets are grouped by `payload.ticketId`
/// and returned sorted newest-first.
pub async fn list_tickets(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TicketSummary>>, StatusCode> {
    let docs = document::Entity::find()
        .filter(document::Column::Type.eq("support_thread"))
        .filter(document::Column::DeletedAt.is_null())
        .order_by_desc(document::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(|e| {
            error!("[Support] DB error listing tickets: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Group by ticketId, keeping insertion order (newest first per group)
    let mut ticket_map: HashMap<String, Vec<document::Model>> = HashMap::new();
    for doc in docs {
        let tid = doc.payload["ticketId"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        ticket_map.entry(tid).or_default().push(doc);
    }

    let mut summaries: Vec<TicketSummary> = ticket_map
        .into_iter()
        .map(|(ticket_id, threads)| {
            // threads[0] is the latest (sorted desc above)
            let latest = &threads[0];
            let meta = &latest.payload["ticket"];

            let subject = meta["subject"]
                .as_str()
                .unwrap_or("(no subject)")
                .to_string();

            let status = meta["status"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();

            let customer = meta["contact"]["fullName"]
                .as_str()
                .or_else(|| meta["contactId"].as_str())
                .or_else(|| latest.payload["from"].as_str())
                .unwrap_or("")
                .to_string();

            let latest_update = latest.payload["createdTime"]
                .as_str()
                .unwrap_or("")
                .to_string();

            TicketSummary {
                ticket_id,
                subject,
                status,
                customer,
                thread_count: threads.len(),
                latest_update,
            }
        })
        .collect();

    // Sort newest-first by the createdTime string (ISO 8601 sorts lexicographically)
    summaries.sort_by(|a, b| b.latest_update.cmp(&a.latest_update));

    Ok(Json(summaries))
}

/// GET /api/support/tickets/:ticket_id/threads
///
/// Returns all thread documents for the given `ticketId`, sorted oldest-first
/// so the UI can display them in conversation order.
pub async fn get_ticket_threads(
    State(state): State<Arc<AppState>>,
    Path(ticket_id): Path<String>,
) -> Result<Json<Vec<document::Model>>, StatusCode> {
    let docs = document::Entity::find()
        .filter(document::Column::Type.eq("support_thread"))
        .filter(document::Column::DeletedAt.is_null())
        .order_by_asc(document::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(|e| {
            error!("[Support] DB error fetching threads for {}: {}", ticket_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let filtered: Vec<document::Model> = docs
        .into_iter()
        .filter(|doc| doc.payload["ticketId"].as_str() == Some(ticket_id.as_str()))
        .collect();

    Ok(Json(filtered))
}

/// POST /api/support/tickets/:ticket_id/summary
///
/// Fetches all threads for the ticket, strips HTML tags, concatenates the plain
/// text, and asks Gemini to produce a concise technical summary.
/// Returns `{"summary": "<text>"}`.
/// Returns 503 if AI is not configured.
pub async fn summarize_ticket(
    State(state): State<Arc<AppState>>,
    Path(ticket_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ai = state.ai_client.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "AI not configured (GEMINI_API_KEY missing)".to_string(),
        )
    })?;

    // Fetch and filter threads (reuse same logic as get_ticket_threads)
    let docs = document::Entity::find()
        .filter(document::Column::Type.eq("support_thread"))
        .filter(document::Column::DeletedAt.is_null())
        .order_by_asc(document::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(|e| {
            error!("[Support] DB error in summarize_ticket {}: {}", ticket_id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    let threads: Vec<&document::Model> = docs
        .iter()
        .filter(|doc| doc.payload["ticketId"].as_str() == Some(ticket_id.as_str()))
        .collect();

    if threads.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("No threads found for ticket {}", ticket_id),
        ));
    }

    // Strip HTML tags and build a labelled transcript
    let tag_re = regex::Regex::new(r"<[^>]*>").expect("static HTML regex");
    let mut parts: Vec<String> = Vec::with_capacity(threads.len());
    for thread in &threads {
        let direction = thread.payload["direction"].as_str().unwrap_or("?");
        let from      = thread.payload["from"].as_str().unwrap_or("?");
        let time      = thread.payload["createdTime"].as_str().unwrap_or("?");
        let raw       = thread.payload["content"].as_str().unwrap_or("");
        let plain     = tag_re.replace_all(raw, " ");
        let clean: String = plain.split_whitespace().collect::<Vec<_>>().join(" ");
        if !clean.is_empty() {
            parts.push(format!("[{direction} | From: {from} | {time}]\n{clean}"));
        }
    }

    if parts.is_empty() {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "All threads have empty content".to_string()));
    }

    let transcript = parts.join("\n\n---\n\n");

    let system_prompt = "You are a technical support assistant. Summarize the following customer \
        support email thread. Extract the core hardware or software problem, any troubleshooting \
        steps already attempted, and the current status. Be concise and professional. \
        Format the result in 2-3 short paragraphs.";

    let summary = ai
        .generate_content(system_prompt, &transcript)
        .await
        .map_err(|e| {
            error!("[Support] AI summary failed for ticket {}: {}", ticket_id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("AI error: {}", e))
        })?;

    info!("[Support] AI summary generated for ticket {} ({} chars)", ticket_id, summary.len());

    Ok(Json(serde_json::json!({ "summary": summary })))
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ImportThreadRequest {
    #[serde(rename = "ticketId")]
    pub ticket_id: String,
    pub threads: Vec<ThreadData>,
    /// Full ticket metadata — stored as-is in the document payload
    pub ticket: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct ThreadData {
    pub id: String,
    pub content: Option<String>,
    pub from: Option<String>,
    pub direction: Option<String>,
    #[serde(rename = "createdTime")]
    pub created_time: Option<String>,
    pub attachments: Option<Vec<AttachmentRef>>,
}

#[derive(Deserialize)]
pub struct AttachmentRef {
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub href: String,
}

#[derive(Serialize)]
pub struct ImportThreadResponse {
    pub success: bool,
    pub imported: usize,
    pub documents: Vec<String>,
    pub errors: Vec<String>,
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// POST /api/support/import-thread
///
/// Imports one or more Zoho ticket threads into the generic `documents` table
/// (type = "support_thread").  Attachments are downloaded via the Node.js
/// Playwright scraper (which carries the Zoho session cookies) and stored in
/// the CAS (`file_resources`), then linked via `entity_attachments`.
pub async fn import_thread(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ImportThreadRequest>,
) -> Result<Json<ImportThreadResponse>, (StatusCode, String)> {
    let mut imported_ids: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for thread in &req.threads {
        let doc_id = Uuid::new_v4();
        let now = Utc::now();

        let payload = serde_json::json!({
            "ticketId":    req.ticket_id,
            "threadId":    thread.id,
            "from":        thread.from,
            "direction":   thread.direction,
            "createdTime": thread.created_time,
            "content":     thread.content,
            "ticket":      req.ticket,
        });

        let new_doc = document::ActiveModel {
            id: Set(doc_id),
            r#type: Set("support_thread".to_string()),
            status: Set("imported".to_string()),
            payload: Set(payload),
            device_id: Set("scraper".to_string()),
            user_id: Set("system".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };

        match new_doc.insert(&state.db).await {
            Ok(doc) => {
                info!(
                    "[Support] Imported thread {} → document {}",
                    thread.id, doc.id
                );

                if let Some(ref atts) = thread.attachments {
                    for att_ref in atts {
                        match download_and_save_attachment(
                            &state,
                            &att_ref.href,
                            &att_ref.file_name,
                            &doc.id.to_string(),
                        )
                        .await
                        {
                            Ok(_) => info!(
                                "[Support] Saved attachment '{}' for document {}",
                                att_ref.file_name, doc.id
                            ),
                            Err(e) => {
                                warn!(
                                    "[Support] Attachment '{}' failed: {}",
                                    att_ref.file_name, e
                                );
                                errors.push(format!(
                                    "Attachment '{}' failed: {}",
                                    att_ref.file_name, e
                                ));
                            }
                        }
                    }
                }

                imported_ids.push(doc.id.to_string());
            }
            Err(e) => {
                let msg = format!("Failed to insert thread '{}': {}", thread.id, e);
                warn!("[Support] {}", msg);
                errors.push(msg);
            }
        }
    }

    Ok(Json(ImportThreadResponse {
        success: !imported_ids.is_empty(),
        imported: imported_ids.len(),
        documents: imported_ids,
        errors,
    }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Calls the Playwright scraper to fetch one attachment file using Zoho session
/// cookies, then saves it to CAS and links it to the given document.
async fn download_and_save_attachment(
    state: &Arc<AppState>,
    href: &str,
    file_name: &str,
    doc_id: &str,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("HTTP client build error: {}", e))?;

    let body = serde_json::json!({
        "href":      href,
        "fileName":  file_name,
        "_from_env": true,
    });

    let resp = client
        .post(format!("{}/api/zoho/download-attachment", SCRAPER_BASE))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Scraper request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Scraper returned HTTP {}", resp.status()));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Scraper response parse error: {}", e))?;

    if data["success"].as_bool() != Some(true) {
        return Err(format!("Scraper reported failure: {}", data));
    }

    let base64_str = data["base64"]
        .as_str()
        .ok_or_else(|| "Missing 'base64' field in scraper response".to_string())?;

    let mime_type = data["mimeType"]
        .as_str()
        .unwrap_or("application/octet-stream");

    let content = base64::engine::general_purpose::STANDARD
        .decode(base64_str)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    // Save to Content-Addressable Storage
    let file_res = state
        .file_store
        .save_file(
            &state.db,
            &content,
            file_name,
            mime_type,
            "scraper",
            &format!("support_thread:{}", doc_id),
            None,
        )
        .await?;

    // Link file to document via entity_attachments
    let new_att = attachment::ActiveModel {
        id: Set(Uuid::new_v4()),
        file_resource_id: Set(file_res.id),
        res_model: Set("document".to_string()),
        res_id: Set(doc_id.to_string()),
        is_main: Set(false),
        tags: Set(Some("support_attachment".to_string())),
        comment: Set(Some(file_name.to_string())),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        deleted_at: Set(None),
    };

    new_att
        .insert(&state.db)
        .await
        .map_err(|e| format!("Failed to create attachment link: {}", e))?;

    Ok(())
}
