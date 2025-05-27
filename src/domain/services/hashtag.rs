use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait SubscribedHashtagService: 'static + Sync + Send {
    fn list_hashtags(&self) -> Result<Vec<String>, Box<dyn Error>>;
    async fn suggest_hashtag(&self, key: &str) -> Result<(), Box<dyn Error>>;
}
