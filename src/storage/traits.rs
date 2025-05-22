use megalodon::entities::Status;
use std::error::Error;

pub trait StatusesIndexer {
    fn insert_statuses(&self, statuses: Vec<&Status>) -> Result<(), Box<dyn Error>>;
}

pub trait HashtagsCollection {
    fn popular_tags(
        &self,
        duration_days: &u16,
        limit: &u16,
    ) -> Result<Vec<(String, u32)>, Box<dyn Error>>;
}

pub trait DataAccessLayer: 'static + Sync + Send + StatusesIndexer + HashtagsCollection {}
