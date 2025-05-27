use async_trait::async_trait;

use megalodon::entities::Status;
use std::collections::HashMap;
use std::error::Error;

#[async_trait]
pub trait StatusService: 'static + Sync + Send {
    async fn paginate_timeline(&self, hashtag: String) -> Vec<Status>;

    /// Persist statuses to avoid hitting the public API constantly
    async fn persist_statuses(&self, statuses: &Vec<Status>) -> Result<(), Box<dyn Error>>;

    /// Retrieve all statuses for the specified hashtags
    async fn retrieve_statuses(
        &self,
        hashtags: Option<&Vec<String>>,
    ) -> Result<Vec<Status>, Box<dyn Error>>;
    /// Retrieve the list of popular tags from the indexed statuses
    fn popular_tags(
        &self,
        periods: Vec<u16>,
        limit: u16,
    ) -> Result<HashMap<u16, Vec<(String, u32)>>, Box<dyn Error>>;
}
