use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::db::AppState;
use crate::services::printer::{generate_labels_pdf, LabelConfig};

/// POST /api/print/labels - Generates PDF labels
/// Mirrors Go's `generateLabels` from `internal/handlers/print.go`.
pub async fn generate_labels(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<LabelConfig>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let label_type = payload.label_type.clone();
    let start_number = payload.start_number;

    let pdf_bytes = generate_labels_pdf(payload).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to generate PDF: {}", e),
        )
    })?;

    let filename = format!("labels_{}_{}.pdf", label_type, start_number);

    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename),
            ),
            (header::CONTENT_LENGTH, pdf_bytes.len().to_string()),
        ],
        pdf_bytes,
    ))
}
