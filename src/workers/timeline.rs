use crate::container::Container;
use crate::domain::services::hashtag::SubscribedHashtagService;
use crate::domain::services::status::{StatusService, StatusServiceError};
use crate::workers::tracker::Worker;
use async_trait::async_trait;
use log::debug;
use megalodon::entities::Status;
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

        let mut tasks: JoinSet<Result<(String, Vec<Status>), StatusServiceError>> = JoinSet::new();
        for hashtag in &hashtags {
            let svc = self.status_service.clone();
            let hashtag_ = hashtag.clone();
            tasks.spawn(async move {
                svc.paginate_timeline(&hashtag_)
                    .await
                    .map(|statuses| (hashtag_.clone(), statuses))
            });
        }

        let mut statuses = vec![];
        while let Some(task) = tasks.join_next().await {
            match task {
                Ok(res) => match res {
                    Ok((hashtag, h_statuses)) => {
                        debug!(
                            "Retrieved {} statuses for tag {}",
                            h_statuses.len(),
                            &hashtag
                        );
                        statuses.extend(h_statuses);
                    }
                    Err(err) => Err(err)?,
                },
                Err(err) if err.is_panic() => panic::resume_unwind(err.into_panic()),
                Err(err) => Err(err)?,
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
        match self.fetch_new_statuses().await {
            Ok(_) => {}
            Err(err) => log::error!("failed to fetch new statuses: {}", err),
        };
        loop {
            tokio::select! {
                _ = sleep(self.update_frequency) => {
                    match self.fetch_new_statuses().await {
                        Ok(_) => {}
                        Err(err) => log::error!("failed to fetch new statuses: {}", err),
                    };
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
