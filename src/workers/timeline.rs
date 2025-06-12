use crate::container::Container;
use crate::domain::services::hashtag::SubscribedHashtagService;
use crate::domain::services::status::StatusService;
use crate::workers::tracker::Worker;
use async_trait::async_trait;
use log::debug;
use std::cmp::Reverse;
use std::error::Error;
use std::ops::Deref;
use std::panic;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

pub struct TimelineUpdater {
    update_frequency: Duration,
    status_service: Arc<dyn StatusService>,
    subscribed_hashtag_service: Arc<dyn SubscribedHashtagService>,
}

impl TimelineUpdater {
    pub fn new(container: Arc<Container>) -> Self {
        Self {
            update_frequency: container
                .settings
                .application
                .timeline_update_frequency
                .deref()
                .clone(),
            status_service: container.status_service.clone(),
            subscribed_hashtag_service: container.subscribed_hashtag_service.clone(),
        }
    }

    async fn fetch_new_statuses(&self) -> Result<(), Box<dyn Error>> {
        let hashtags = self.subscribed_hashtag_service.list_hashtags()?;

        let mut tasks = JoinSet::new();
        for hashtag in &hashtags {
            let svc = self.status_service.clone();
            let hashtag_ = hashtag.clone();
            tasks.spawn(async move { (hashtag_.clone(), svc.paginate_timeline(hashtag_).await) });
        }

        let mut statuses = vec![];
        while let Some(res) = tasks.join_next().await {
            match res {
                Ok((hashtag, h_statuses)) => {
                    debug!(
                        "Retrieved {} statuses for tag {}",
                        h_statuses.len(),
                        &hashtag
                    );
                    statuses.extend(h_statuses);
                }
                Err(err) if err.is_panic() => panic::resume_unwind(err.into_panic()),
                Err(err) => panic!("{err}"),
            }
        }

        // https://docs.joinmastodon.org/api/guidelines/#id
        statuses.sort_by_key(|status| Reverse((status.id.len(), status.id.clone())));
        statuses.dedup_by_key(|status| status.id.clone());

        debug!("{} statuses after deduplication", statuses.len());

        self.status_service.persist_statuses(&statuses).await?;
        Ok(())
    }
}

#[async_trait]
impl Worker for TimelineUpdater {
    async fn run(&self, cancellation_token: CancellationToken) {
        log::info!("starting timeline updater worker");
        self.fetch_new_statuses().await.unwrap();
        loop {
            tokio::select! {
                _ = sleep(self.update_frequency) => {
                    self.fetch_new_statuses().await.unwrap();
                    continue;
                }

                _ = cancellation_token.cancelled() => {
                    log::info!("gracefully shutting down the timeline updater");
                    break;
                }
            }
        }
    }
}
