use axum::{extract::State, http::StatusCode, Json};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use crate::db::AppState;
use crate::models::file_resource;

// --- DTOs ---

#[derive(Deserialize)]
pub struct AnalyzeImageRequest {
    pub file_id: String,
    pub prompt: Option<String>,
}

#[derive(Serialize)]
pub struct AnalyzeImageResponse {
    pub success: bool,
    pub analysis: serde_json::Value,
    pub file_id: String,
}

#[derive(Deserialize)]
pub struct AIResponseRequest {
    #[serde(rename = "interactionId")]
    pub interaction_id: String,
    pub response: String,
    pub barcode: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
}

// --- Handlers ---

const IMAGE_ANALYSIS_SYSTEM_PROMPT: &str = r#"ROLE: Warehouse Logistics AI.
TASK: Analyze the input.
OUTPUT: JSON only.
JSON STRUCTURE:
{"condition":"good"|"damaged"|"unknown","labels_visible":true|false,"description":"...","ocr_text":"...","tags":["box","brown","label"],"action_recommendation":"..."}"#;

/// POST /api/ai/analyze-image — analyzes an image using Gemini multimodal.
/// Mirrors Go's image analysis in `internal/handlers/ai_handlers.go`.
pub async fn analyze_image(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalyzeImageRequest>,
) -> Result<Json<AnalyzeImageResponse>, (StatusCode, String)> {
    let ai = state.ai_client.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "AI module not initialized (missing GEMINI_API_KEY)".to_string(),
    ))?;

    let file_uuid = uuid::Uuid::parse_str(&payload.file_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid file UUID".to_string()))?;

    let file_res = file_resource::Entity::find_by_id(file_uuid)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "File not found".to_string()))?;

    let content = file_res.avatar_data.ok_or((
        StatusCode::NOT_IMPLEMENTED,
        "Only inline avatar data supported for analysis (no file bytes)".to_string(),
    ))?;

    let user_prompt = payload
        .prompt
        .unwrap_or_else(|| "Analyze this warehouse image.".to_string());

    let raw_resp = ai
        .generate_content_from_image(
            IMAGE_ANALYSIS_SYSTEM_PROMPT,
            &user_prompt,
            &file_res.mime_type,
            &content,
        )
        .await
        .map_err(|e| {
            error!("AI Vision Error: {}", e);
            (StatusCode::BAD_GATEWAY, format!("AI error: {}", e))
        })?;

    let clean_json = crate::utils::json::sanitize_json(&raw_resp);

    let analysis: serde_json::Value =
        serde_json::from_str(&clean_json).unwrap_or_else(|_| {
            serde_json::json!({
                "raw_analysis": clean_json,
                "condition": "unknown"
            })
        });

    Ok(Json(AnalyzeImageResponse {
        success: true,
        analysis,
        file_id: payload.file_id,
    }))
}

/// POST /api/ai/respond — processes user feedback from Android app.
/// Mirrors Go's `handleAiRespond` from `internal/handlers/scan.go`.
pub async fn handle_ai_respond(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<AIResponseRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!(
        "AI Response received: {} for {} (Interaction: {})",
        payload.response, payload.barcode, payload.interaction_id
    );

    let action_taken = if payload.response.eq_ignore_ascii_case("yes") {
        // TODO: Use ToolService to link alias in DB (Phase 6.3)
        "linked_to_db"
    } else {
        "logged_only"
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "AI response processed",
        "action": action_taken,
    })))
}
