use axum::Router;
use axum::{extract::DefaultBodyLimit, routing::post};
use tracing::info;

mod controllers;
mod error;
mod settings;

pub use error::{AppError, AppResult, DomainError};

#[tokio::main]
async fn main() {
    let settings = settings::Settings::new().expect("failed to load settings");
    let bind_address = settings.bind_address.clone();

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/utils/mark", post(controllers::utils::mark))
        .layer(DefaultBodyLimit::max(settings.utils.mark_pdf_max_size_byte))
        .with_state(settings);

    let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();
    info!("Listening on {}", bind_address);
    axum::serve(listener, app).await.unwrap();
}
