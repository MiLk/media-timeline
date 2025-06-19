use crate::infrastructure::error::DbError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use megalodon::entities::Status;

#[async_trait]
pub trait RecentStatusRepository: 'static + Sync + Send {
    fn get_recent_status_id(&self, key: &str) -> Result<Option<String>, DbError>;
    fn set_recent_status_id(&self, key: &String, value: &String) -> Result<(), DbError>;
}

pub trait StatusIndexRepository: 'static + Sync + Send {
    fn insert_statuses(&self, statuses: Vec<&Status>) -> Result<(), DbError>;

    fn search_statuses(
        &self,
        hashtags: Option<&Vec<String>>,
        limit: u16,
    ) -> Result<Vec<String>, DbError>;

    fn popular_statuses(
        &self,
        hashtags_o: Option<&Vec<String>>,
        since: DateTime<Utc>,
        limit: u16,
    ) -> Result<Vec<String>, DbError>;

    fn list_stale_statuses(
        &self,
        since: DateTime<Utc>,
        fresh_since: DateTime<Utc>,
        limit: u16,
    ) -> Result<Vec<String>, DbError>;

    fn popular_tags(&self, duration_days: &u16, limit: &u16)
    -> Result<Vec<(String, u32)>, DbError>;
}
