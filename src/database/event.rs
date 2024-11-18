use chrono::{DateTime, Utc};
use sqlx::{types::Uuid, PgPool};
use ulid::Ulid;

use crate::AppState;

pub trait AsEventAccessor {
  fn event_accessor(&self) -> EventAccessor;
}

impl AsEventAccessor for AppState {
  fn event_accessor(&self) -> EventAccessor {
    EventAccessor {
      database: &self.database,
    }
  }
}

pub struct EventAccessor<'db> {
  pub database: &'db PgPool,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Event {
  pub id: Uuid,
  pub title: String,
  pub start_at: DateTime<Utc>,
  pub end_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub end_booking_at: DateTime<Utc>,
  pub start_booking_at: DateTime<Utc>,
  pub image_url: Option<String>,
  pub description: String,
}

#[bon::bon]
impl<'db> EventAccessor<'db> {
  pub async fn list(&self) -> Result<Vec<Event>, sqlx::Error> {
    sqlx::query_as!(Event, "SELECT * FROM event")
      .fetch_all(self.database)
      .await
  }

  pub async fn get(&self, id: Uuid) -> Result<Option<Event>, sqlx::Error> {
    sqlx::query_as!(Event, "SELECT * FROM event WHERE id = $1", id)
      .fetch_optional(self.database)
      .await
  }

  #[builder]
  pub async fn create(
    &self,
    title: String,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    start_booking_at: DateTime<Utc>,
    end_booking_at: DateTime<Utc>,
    image_url: Option<String>,
    description: String,
  ) -> Result<Event, sqlx::Error> {
    let id: Uuid = Ulid::new().into();
    let event = sqlx::query_as!(
      Event,
      "INSERT INTO event
      (id, title, start_at, end_at, created_at, updated_at, start_booking_at, end_booking_at, image_url, description)
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *",
      id,
      title,
      start_at,
      end_at,
      Utc::now(),
      Utc::now(),
      start_booking_at,
      end_booking_at,
      image_url,
      description
    )
    .fetch_one(self.database)
    .await?;
    Ok(event)
  }

  #[builder]
  pub async fn update(
    &self,
    id: Ulid,
    title: String,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    start_booking_at: DateTime<Utc>,
    end_booking_at: DateTime<Utc>,
    image_url: Option<String>,
    description: String,
  ) -> Result<Event, sqlx::Error> {
    let event = sqlx::query_as!(
      Event,
      "UPDATE event
      SET title = $2, start_at = $3, end_at = $4, start_booking_at = $5,
          end_booking_at = $6, image_url = $7, description = $8, updated_at = $9
      WHERE id = $1 RETURNING *",
      Uuid::from(id),
      title,
      start_at,
      end_at,
      start_booking_at,
      end_booking_at,
      image_url,
      description,
      Utc::now()
    )
    .fetch_one(self.database)
    .await?;
    Ok(event)
  }

  pub async fn delete(&self, id: Ulid) -> Result<Event, sqlx::Error> {
    let event = sqlx::query_as!(Event, "DELETE FROM event WHERE id = $1 RETURNING *", Uuid::from(id))
      .fetch_one(self.database)
      .await?;
    Ok(event)
  }
}
