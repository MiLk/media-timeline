use async_trait::async_trait;

use chrono::{DateTime, Utc};
use megalodon::entities::Status;
use std::collections::HashMap;
use std::error::Error;

#[async_trait]
pub trait StatusService: 'static + Sync + Send {
    async fn paginate_timeline(&self, hashtag: String) -> Vec<Status>;

    async fn fetch_statuses(&self, ids: &[String]) -> Result<Vec<Status>, Box<dyn Error>>;

    /// Persist statuses to avoid hitting the public API constantly
    async fn persist_statuses(&self, statuses: &Vec<Status>) -> Result<(), Box<dyn Error>>;

    /// Retrieve all statuses for the specified hashtags
    async fn retrieve_statuses(
        &self,
        hashtags: Option<&Vec<String>>,
        limit: u16,
    ) -> Result<Vec<Status>, Box<dyn Error>>;

    async fn popular_statuses(
        &self,
        hashtags: Option<&Vec<String>>,
        since: DateTime<Utc>,
        limit: u16,
    ) -> Result<Vec<Status>, Box<dyn Error>>;

    // List ID for statuses created after `since` but refreshed before `fresh_since`.
    async fn list_stale_statuses(
        &self,
        since: DateTime<Utc>,
        fresh_since: DateTime<Utc>,
    ) -> Result<Vec<String>, Box<dyn Error>>;

    /// Retrieve the list of popular tags from the indexed statuses
    fn popular_tags(
        &self,
        periods: Vec<u16>,
        limit: u16,
    ) -> Result<HashMap<u16, Vec<(String, u32)>>, Box<dyn Error>>;
}
