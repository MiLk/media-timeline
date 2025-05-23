use crate::storage::HashtagKey;
use megalodon::entities::Status;
use std::collections::HashMap;
use std::error::Error;

pub trait StatusesIndexer {
    fn insert_statuses(&self, statuses: Vec<&Status>) -> Result<(), Box<dyn Error>>;
}

pub trait StatusTagsCollection {
    fn popular_tags(
        &self,
        duration_days: &u16,
        limit: &u16,
    ) -> Result<Vec<(String, u32)>, Box<dyn Error>>;
}

pub trait DataAccessLayer: 'static + Sync + Send + StatusesIndexer + StatusTagsCollection {}

pub trait RecentStatusesService {
    fn get_recent_status_id(&self, key: &str) -> Option<String>;
    fn set_recent_status_id(&self, key: &String, value: &String) -> Option<String>;
}

pub trait StatusesService {
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

pub trait SubscribedHashtagsService {
    fn list_hashtags(&self) -> Vec<HashtagKey>;
    async fn suggest_hashtag(&self, key: &str) -> ();
}
