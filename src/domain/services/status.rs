use async_trait::async_trait;

use actix_web::ResponseError;
use chrono::{DateTime, Utc};
use megalodon::entities::Status;
use std::collections::HashMap;
use std::io;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum StatusServiceError {
    #[error("unable to read the file")]
    FileError(#[from] io::Error),
    #[error("unable to parse the content")]
    ParsingError(#[from] serde_json::Error),

    #[error("Unable to retrieve statuses from Mastodon API")]
    CantRetrieveStatuses(#[from] megalodon::error::Error),
    #[error("Unable to update the recent status ID locally")]
    CantUpdateStatuses,

    #[error(transparent)]
    DbError(#[from] crate::infrastructure::error::DbError),

    #[error(transparent)]
    TaskFailed(#[from] JoinError),
}

impl ResponseError for StatusServiceError {}

#[async_trait]
pub trait StatusService: 'static + Sync + Send {
    async fn paginate_timeline(&self, hashtag: &String) -> Result<Vec<Status>, StatusServiceError>;

    async fn fetch_statuses(&self, ids: &[String]) -> Result<Vec<Status>, StatusServiceError>;

    /// Persist statuses to avoid hitting the public API constantly
    async fn persist_statuses(&self, statuses: &Vec<Status>) -> Result<(), StatusServiceError>;

    /// Retrieve all statuses for the specified hashtags
    async fn retrieve_statuses(
        &self,
        hashtags: Option<&Vec<String>>,
        limit: u16,
    ) -> Result<Vec<Status>, StatusServiceError>;

    async fn popular_statuses(
        &self,
        hashtags: Option<&Vec<String>>,
        since: DateTime<Utc>,
        limit: u16,
    ) -> Result<Vec<Status>, StatusServiceError>;

    // List ID for statuses created after `since` but refreshed before `fresh_since`.
    async fn list_stale_statuses(
        &self,
        since: DateTime<Utc>,
        fresh_since: DateTime<Utc>,
        limit: u16,
    ) -> Result<Vec<String>, StatusServiceError>;

    /// Retrieve the list of popular tags from the indexed statuses
    fn popular_tags(
        &self,
        periods: Vec<u16>,
        limit: u16,
    ) -> Result<HashMap<u16, Vec<(String, u32)>>, StatusServiceError>;
}
