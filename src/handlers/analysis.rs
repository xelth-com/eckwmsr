use axum::{extract::State, http::StatusCode, Json};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;

use crate::{db::AppState, models::document};

#[derive(Serialize)]
pub struct TicketDump {
    pub ticket_number: String,
    pub subject: String,
    pub status: String,
    pub text_content: String,
}

/// GET /api/analysis/support-dump
/// Fetches support threads, strips HTML, and groups them by ticket for AI analysis.
pub async fn support_dump(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TicketDump>>, StatusCode> {
    let docs = document::Entity::find()
        .filter(document::Column::Type.eq("support_thread"))
        .filter(document::Column::DeletedAt.is_null())
        .all(&state.db)
        .await
        .map_err(|e| {
            error!("DB error in support_dump: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Group by ticketId
    let mut ticket_map: HashMap<String, Vec<&document::Model>> = HashMap::new();
    for doc in &docs {
        let tid = doc.payload["ticketId"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        ticket_map.entry(tid).or_default().push(doc);
    }

    let tag_re = regex::Regex::new(r"<[^>]*>").expect("static HTML regex");
    let mut dump = Vec::new();

    for (_, mut threads) in ticket_map {
        // Sort threads by creation time (oldest first) to form a chronological conversation
        threads.sort_by(|a, b| {
            let t_a = a.payload["createdTime"].as_str().unwrap_or("");
            let t_b = b.payload["createdTime"].as_str().unwrap_or("");
            t_a.cmp(t_b)
        });

        let meta = &threads[0].payload["ticket"];
        let subject = meta["subject"].as_str().unwrap_or("").to_string();
        let status = meta["status"].as_str().unwrap_or("").to_string();
        let ticket_number = meta["ticketNumber"].as_str().unwrap_or("").to_string();

        let mut all_text = String::new();
        for t in threads {
            let dir = t.payload["direction"].as_str().unwrap_or("?");
            let raw = t.payload["content"].as_str().unwrap_or("");

            // Strip HTML
            let plain = tag_re.replace_all(raw, " ");
            let clean = plain.split_whitespace().collect::<Vec<_>>().join(" ");

            if !clean.is_empty() {
                all_text.push_str(&format!("[{}] {}\n", dir.to_uppercase(), clean));
            }
        }

        dump.push(TicketDump {
            ticket_number,
            subject,
            status,
            text_content: all_text.trim().to_string(),
        });
    }

    // Sort by ticket number descending
    dump.sort_by(|a, b| b.ticket_number.cmp(&a.ticket_number));

    Ok(Json(dump))
}
