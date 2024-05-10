use axum::body::Bytes;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use pdf_watermark::mark_pdf;
use pdfium_render::points::PdfPoints;
use serde::Deserialize;

#[derive(Default, Deserialize)]
struct MarkQuery {
    text: String,
    font_size: f32,
    padding_w: f32,
    padding_h: f32,
    rot_deg: f32,
}

async fn mark(query: Query<MarkQuery>, pdf: Bytes) -> Result<Vec<u8>, AppError> {
    println!("request received");
    let marked = mark_pdf(
        &pdf,
        &query.text,
        query.font_size,
        PdfPoints::from_mm(query.padding_w),
        PdfPoints::from_mm(query.padding_h),
        query.rot_deg,
    )?;
    Ok(marked)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", post(mark));

    let listener = tokio::net::TcpListener::bind("[::]:3050").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
