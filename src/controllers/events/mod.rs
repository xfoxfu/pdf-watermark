use crate::database::AsEventAccessor;
use crate::{AppError, AppResult, AppState};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use ulid::Ulid;

#[derive(Debug, Serialize, Clone)]
pub struct Event {
  pub id: Ulid,
  pub title: String,
  pub start_at: chrono::DateTime<chrono::Utc>,
  pub end_at: chrono::DateTime<chrono::Utc>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
  pub start_booking_at: chrono::DateTime<chrono::Utc>,
  pub end_booking_at: chrono::DateTime<chrono::Utc>,
  pub image_url: Option<String>,
  pub description: String,
}

impl From<crate::database::Event> for Event {
  fn from(e: crate::database::Event) -> Self {
    Self {
      id: e.id.into(),
      title: e.title,
      start_at: e.start_at,
      end_at: e.end_at,
      created_at: e.created_at,
      updated_at: e.updated_at,
      start_booking_at: e.start_booking_at,
      end_booking_at: e.end_booking_at,
      image_url: e.image_url,
      description: e.description,
    }
  }
}

pub async fn list(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
  let events = state
    .event_accessor()
    .list()
    .await?
    .into_iter()
    .map(Into::into)
    .collect::<Vec<Event>>();
  Ok(Json(events))
}

pub async fn get(State(state): State<AppState>, Path(id): Path<Ulid>) -> AppResult<impl IntoResponse> {
  let Some(event) = state.event_accessor().get(id.into()).await?.map(Event::from) else {
    return Err(AppError::DomainError(crate::DomainError::EventNotFound { id }));
  };
  Ok(Json(event))
}
