use async_trait::async_trait;
use megalodon::entities::Status;
use std::error::Error;

#[async_trait]
pub trait RecentStatusRepository: 'static + Sync + Send {
    fn get_recent_status_id(&self, key: &str) -> Result<Option<String>, Box<dyn Error>>;
    fn set_recent_status_id(&self, key: &String, value: &String) -> Result<(), Box<dyn Error>>;
}

pub trait StatusIndexRepository: 'static + Sync + Send {
    fn insert_statuses(&self, statuses: Vec<&Status>) -> Result<(), Box<dyn Error>>;

    fn search_statuses(
        &self,
        hashtags: Option<&Vec<String>>,
        limit: u16,
    ) -> Result<Vec<String>, Box<dyn Error>>;

    fn popular_tags(
        &self,
        duration_days: &u16,
        limit: &u16,
    ) -> Result<Vec<(String, u32)>, Box<dyn Error>>;
}
