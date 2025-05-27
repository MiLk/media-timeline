use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait SubscribedHashtagRepository: 'static + Sync + Send {
    fn increment_vote(&self, key: &str) -> Result<(), Box<dyn Error>>;
    fn list(&self) -> Result<Vec<String>, Box<dyn Error>>;
}
