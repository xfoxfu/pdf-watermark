use chrono::{DateTime, Utc};
use sqlx::{types::Uuid, PgPool};

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

impl<'db> EventAccessor<'db> {
    pub async fn list(&self) -> Result<Vec<Event>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, title, start_at, end_at, created_at, updated_at, 
                end_booking_at, start_booking_at, image_url, description
            FROM event",
        )
        .fetch_all(self.database)
        .await
    }
}
