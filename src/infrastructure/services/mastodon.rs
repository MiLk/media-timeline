use log::debug;
use megalodon::error::Error;
use megalodon::mastodon::Mastodon;
use megalodon::megalodon::GetHomeTimelineInputOptions;
use megalodon::{Megalodon, entities};

#[derive(Debug, Clone)]
pub struct MastodonClient {
    client: Mastodon,
}

impl MastodonClient {
    pub fn new(base_url: String, user_agent: Option<String>) -> Result<MastodonClient, Error> {
        debug!("Using the following User-Agent: {:?}", user_agent);
        let client = Mastodon::new(base_url, None, user_agent)?;
        Ok(MastodonClient { client })
    }
    pub async fn get_tag_timeline(
        &self,
        hashtag: &String,
        min_id: Option<String>,
    ) -> Result<Vec<entities::Status>, Error> {
        debug!("Getting tag timeline for {} from {:?}", hashtag, min_id);
        self.client
            .get_tag_timeline(
                hashtag.clone(),
                Some(&GetHomeTimelineInputOptions {
                    only_media: Some(true),
                    limit: Some(40),
                    max_id: None,
                    since_id: None,
                    min_id,
                    local: None,
                }),
            )
            .await
            .map(|res| res.json())
    }

    pub async fn get_status(&self, id: String) -> Result<entities::Status, Error> {
        self.client.get_status(id).await.map(|res| res.json())
    }
}
