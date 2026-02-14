use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

// Embeds the frontend build into the binary
#[derive(RustEmbed)]
#[folder = "web/build/"]
struct Assets;

pub async fn static_handler(req: Request<Body>) -> impl IntoResponse {
    let path = req.uri().path().trim_start_matches('/');

    // Default to index.html for root path
    let path = if path.is_empty() {
        "index.html"
    } else {
        path
    };

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data.to_vec()))
                .unwrap()
        }
        None => {
            // Fallback to index.html for SPA client-side routing
            match Assets::get("index.html") {
                Some(content) => Response::builder()
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(Body::from(content.data.to_vec()))
                    .unwrap(),
                None => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("404 Not Found"))
                    .unwrap(),
            }
        }
    }
}
