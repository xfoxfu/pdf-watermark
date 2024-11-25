use axum::{
  extract::{Path, State},
  response::IntoResponse,
  Json,
};
use serde::Deserialize;
use ulid::Ulid;
use utoipa::ToSchema;

use crate::{database::AsEventAccessor, AppError, AppResult, AppState};

use super::Event;

#[utoipa::path(
  get, path = "/events",
  responses((status = 200, body = Event)),
)]
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

#[utoipa::path(
  get, path = "/events/{id}",
  responses((status = 200, body = Event)),
  params(("id" = Ulid, Path, description = "The ID of the event")),
)]
pub async fn get(State(state): State<AppState>, Path(id): Path<Ulid>) -> AppResult<impl IntoResponse> {
  let Some(event) = state.event_accessor().get(id.into()).await?.map(Event::from) else {
    return Err(AppError::DomainError(crate::DomainError::EventNotFound { id }));
  };
  Ok(Json(event))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrUpdateEvent {
  pub title: String,
  pub start_at: chrono::DateTime<chrono::Utc>,
  pub end_at: chrono::DateTime<chrono::Utc>,
  pub start_booking_at: chrono::DateTime<chrono::Utc>,
  pub end_booking_at: chrono::DateTime<chrono::Utc>,
  pub image_url: Option<String>,
  pub description: String,
}

#[utoipa::path(
  post, path = "/events",
  responses((status = 200, body = Event)),
)]
pub async fn create(
  State(state): State<AppState>,
  Json(payload): Json<CreateOrUpdateEvent>,
) -> AppResult<impl IntoResponse> {
  let event = state
    .event_accessor()
    .create()
    .title(payload.title)
    .start_at(payload.start_at)
    .end_at(payload.end_at)
    .start_booking_at(payload.start_booking_at)
    .end_booking_at(payload.end_booking_at)
    .maybe_image_url(payload.image_url)
    .description(payload.description)
    .call()
    .await?;
  Ok(Json(Event::from(event)))
}

#[utoipa::path(
  put, path = "/events/{id}",
  responses((status = 200, body = Event)),
  params(("id" = Ulid, Path, description = "The ID of the event")),
)]
pub async fn update(
  State(state): State<AppState>,
  Path(id): Path<Ulid>,
  Json(payload): Json<CreateOrUpdateEvent>,
) -> AppResult<impl IntoResponse> {
  let event = state
    .event_accessor()
    .update()
    .id(id)
    .title(payload.title)
    .start_at(payload.start_at)
    .end_at(payload.end_at)
    .start_booking_at(payload.start_booking_at)
    .end_booking_at(payload.end_booking_at)
    .maybe_image_url(payload.image_url)
    .description(payload.description)
    .call()
    .await?;
  Ok(Json(Event::from(event)))
}

#[utoipa::path(
  delete,
  path = "/events/{id}",
  responses((status = 200, body = Event)),
  params(("id" = Ulid, Path, description = "The ID of the event")),
)]
pub async fn delete(State(state): State<AppState>, Path(id): Path<Ulid>) -> AppResult<impl IntoResponse> {
  let event = state.event_accessor().delete(id).await?;
  Ok(Json(Event::from(event)))
}
