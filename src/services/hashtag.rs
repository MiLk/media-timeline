use crate::domain::repositories::hashtag::SubscribedHashtagRepository;
use crate::domain::services::hashtag::SubscribedHashtagService;
use async_trait::async_trait;
use log::debug;
use std::error::Error;
use std::sync::Arc;

pub struct SubscribedHashtagServiceImpl {
    repository: Arc<dyn SubscribedHashtagRepository>,
}

impl SubscribedHashtagServiceImpl {
    pub fn new(repository: Arc<dyn SubscribedHashtagRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl SubscribedHashtagService for SubscribedHashtagServiceImpl {
    fn list_hashtags(&self) -> Result<Vec<String>, Box<dyn Error>> {
        self.repository.list()
    }

    async fn suggest_hashtag(&self, key: &str) -> Result<(), Box<dyn Error>> {
        if !key.is_empty() {
            let attributes = self.repository.increment_vote(key)?;
            debug!("Hashtag suggested: {} -> {:?}", key, attributes)
        }
        Ok(())
    }
}
