use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use pdf_watermark::mark_pdf;
use pdfium_render::points::PdfPoints;

#[derive(Default)]
struct MarkRequest {
    pdf: Vec<u8>,
    text: String,
    font_size: f32,
    padding_w: f32,
    padding_h: f32,
    rot_deg: f32,
}

async fn mark(mut form: Multipart) -> Result<Vec<u8>, AppError> {
    let mut req = MarkRequest::default();

    while let Some(field) = form.next_field().await? {
        let field_name = field.name().map(str::to_string);
        let bytes: Vec<_> = field.bytes().await?.into_iter().collect();
        match field_name.as_deref() {
            Some("text") => req.text = String::from_utf8(bytes)?,
            Some("font_size") => req.font_size = String::from_utf8(bytes)?.parse()?,
            Some("padding_w") => req.padding_w = String::from_utf8(bytes)?.parse()?,
            Some("padding_h") => req.padding_h = String::from_utf8(bytes)?.parse()?,
            Some("rot_deg") => req.rot_deg = String::from_utf8(bytes)?.parse()?,
            Some(_) | None => req.pdf = bytes,
        }
    }

    let marked = mark_pdf(
        &req.pdf,
        &req.text,
        req.font_size,
        PdfPoints::from_mm(req.padding_w),
        PdfPoints::from_mm(req.padding_h),
        req.rot_deg,
    )?;
    Ok(marked)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", post(mark));

    let listener = tokio::net::TcpListener::bind("[::]:3000").await.unwrap();
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
