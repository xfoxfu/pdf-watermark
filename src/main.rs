use axum::extract::DefaultBodyLimit;
use axum::routing::*;
use axum::Router;
use sqlx::{MySqlPool, PgPool};
use tracing::info;

mod controllers;
mod database;
mod error;
mod settings;

pub use error::{AppError, AppResult, DomainError};
use utoipa::OpenApi;
use utoipa_scalar::Scalar;
use utoipa_scalar::Servable;

#[derive(OpenApi)]
#[openapi(paths(
  controllers::events::list,
  controllers::events::get,
  controllers::events::create,
  controllers::events::update,
  controllers::events::delete
))]
struct ApiDoc;

#[derive(Clone)]
pub struct AppState {
  pub database: PgPool,
  pub legacy_database: MySqlPool,
  pub settings: settings::Settings,
}

#[tokio::main]
async fn main() {
  let settings = settings::Settings::new().expect("failed to load settings");
  let postgres = PgPool::connect(&settings.database.url)
    .await
    .expect("failed to connect to postgres");
  let mysql = MySqlPool::connect(&settings.database.legacy_url)
    .await
    .expect("failed to connect to mysql");
  let app_state = AppState {
    database: postgres,
    legacy_database: mysql,
    settings: settings.clone(),
  };

  tracing_subscriber::fmt::init();

  let app = Router::new()
    .route(
      "/utils/mark",
      post(controllers::utils::mark).layer(DefaultBodyLimit::max(settings.utils.mark_pdf_max_size_byte)),
    )
    .route("/events", get(controllers::events::list))
    .route("/events", post(controllers::events::create))
    .route("/events/:id", get(controllers::events::get))
    .route("/events/:id", put(controllers::events::update))
    .route("/events/:id", delete(controllers::events::delete))
    .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
    .with_state(app_state);

  let listener = tokio::net::TcpListener::bind(&settings.bind_address).await.unwrap();
  info!("Listening on {}", settings.bind_address);
  axum::serve(listener, app).await.unwrap();
}
