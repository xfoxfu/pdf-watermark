use serde::Serialize;
use ulid::Ulid;
use utoipa::ToSchema;

mod event;
pub use event::*;

#[derive(Debug, Serialize, Clone, ToSchema)]
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
