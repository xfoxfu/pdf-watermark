use axum::routing::post;
use axum::Router;
use tracing::info;

mod controllers;
mod error;
mod settings;

pub use error::{AppError, AppResult, DomainError};

#[tokio::main]
async fn main() {
    let settings = settings::Settings::new().expect("failed to load settings");

    tracing_subscriber::fmt::init();

    let app = Router::new().route("/utils/mark", post(controllers::utils::mark));

    let listener = tokio::net::TcpListener::bind(&settings.bind_address)
        .await
        .unwrap();
    info!("Listening on {}", settings.bind_address);
    axum::serve(listener, app).await.unwrap();
}
