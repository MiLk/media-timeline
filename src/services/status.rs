use crate::domain::repositories::status::{RecentStatusRepository, StatusIndexRepository};
use crate::domain::services::status::StatusService;
use crate::infrastructure::services::mastodon::MastodonClient;
use async_trait::async_trait;
use log::debug;
use megalodon::entities::Status;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::fs::{File, create_dir_all};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn directory_for_status(status_id: &str) -> String {
    let len = status_id.len();
    let dir1 = if len <= 18 {
        "0"
    } else {
        &status_id[0..len - 18]
    };
    let dir2 = if len <= 14 {
        "0"
    } else {
        &status_id[0..len - 14]
    };
    format!("data/statuses/{}/{}", dir1, dir2)
}

pub struct StatusServiceImpl {
    mastodon_client: Arc<MastodonClient>,
    recent_repository: Arc<dyn RecentStatusRepository>,
    index_repository: Arc<dyn StatusIndexRepository>,
}

impl StatusServiceImpl {
    pub(crate) fn new(
        mastodon_client: Arc<MastodonClient>,
        recent_repository: Arc<dyn RecentStatusRepository>,
        index_repository: Arc<dyn StatusIndexRepository>,
    ) -> Self {
        Self {
            mastodon_client,
            recent_repository,
            index_repository,
        }
    }
}

#[async_trait]
impl StatusService for StatusServiceImpl {
    async fn paginate_timeline(&self, hashtag: String) -> Vec<Status> {
        match self
            .recent_repository
            .get_recent_status_id(hashtag.as_str())
            .unwrap_or(None)
        {
            None => {
                let statuses = self
                    .mastodon_client
                    .get_tag_timeline(&hashtag, None)
                    .await
                    .expect("Unable to retrieve statuses from Mastodon API");
                if let Some(status) = statuses.last() {
                    self.recent_repository
                        .set_recent_status_id(&hashtag, &status.id)
                        .expect("Unable to update the recent status ID locally");
                }
                statuses
            }
            Some(recent_id) => {
                let mut statuses: Vec<Status> = vec![];
                let mut last_id = recent_id.clone();
                loop {
                    let page = self
                        .mastodon_client
                        .get_tag_timeline(&hashtag, Some(last_id))
                        .await
                        .expect("Unable to retrieve statuses from Mastodon API");
                    if page.is_empty() {
                        break;
                    }
                    match page
                        .iter()
                        .max_by_key(|&status| (status.id.len(), &status.id))
                    {
                        None => break, // Should already be covered by checking if the page is empty
                        Some(highest_id) => last_id = highest_id.id.clone(),
                    }
                    debug!(
                        "Retrieved {} new statuses for {} - last: {}",
                        page.len(),
                        hashtag,
                        last_id
                    );
                    self.recent_repository
                        .set_recent_status_id(&hashtag, &last_id)
                        .expect("Unable to update the recent status ID locally");
                    statuses.extend(page)
                }
                statuses.sort_by_key(|status| Reverse((status.id.len(), status.id.clone())));
                statuses
            }
        }
    }
    async fn persist_statuses(&self, statuses: &Vec<Status>) -> Result<(), Box<dyn Error>> {
        async fn write_status(
            status: Status,
            index_repository: Arc<dyn StatusIndexRepository>,
        ) -> () {
            let dir = directory_for_status(&status.id.as_str());
            create_dir_all(&dir)
                .await
                .expect(format!("failed to create dir {}", dir).as_str());
            let filepath = format!("{}/{}.json", dir, &status.id);
            let mut file = File::create(&filepath)
                .await
                .expect(format!("failed to create file {}", &filepath).as_str());
            file.write_all(
                serde_json::to_string(&status)
                    .expect(format!("failed to serialize status {}", &status.id).as_str())
                    .as_bytes(),
            )
            .await
            .expect(format!("failed to write to file {}", &filepath).as_str());

            index_repository
                .insert_statuses(vec![&status])
                .expect(format!("failed to insert status {}", &status.id).as_str());
        }

        let mut tasks = Vec::with_capacity(statuses.len());
        for status in statuses.iter() {
            tasks.push(tokio::spawn(write_status(
                status.clone(),
                self.index_repository.clone(),
            )))
        }
        for task in tasks {
            task.await?;
        }
        Ok(())
    }

    async fn retrieve_statuses(
        &self,
        hashtags: Option<&Vec<String>>,
        limit: u16,
    ) -> Result<Vec<Status>, Box<dyn Error>> {
        let status_ids = self.index_repository.search_statuses(hashtags, limit)?;
        let mut statuses = Vec::new();
        for id in status_ids {
            let dir = directory_for_status(&id);
            let filepath = format!("{}/{}.json", dir, &id);
            let mut file = File::open(filepath).await?;
            let mut content = String::new();
            file.read_to_string(&mut content).await?;
            let decoded: Status = serde_json::from_str(content.as_str())?;
            statuses.push(decoded);
        }
        debug!("{} statuses read from storage", statuses.len());
        Ok(statuses)
    }

    fn popular_tags(
        &self,
        periods: Vec<u16>,
        limit: u16,
    ) -> Result<HashMap<u16, Vec<(String, u32)>>, Box<dyn Error>> {
        periods
            .iter()
            .map(|&period| Ok((period, self.index_repository.popular_tags(&period, &limit)?)))
            .collect()
    }
}
