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
    pub fn new(base_url: String) -> Result<MastodonClient, Error> {
        let client = Mastodon::new(base_url, None, None)?;
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
}
