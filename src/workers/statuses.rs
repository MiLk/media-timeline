use crate::container::Container;
use crate::domain::services::status::StatusService;
use crate::settings::StatusRefreshSettings;
use crate::workers::tracker::Worker;
use async_trait::async_trait;
use chrono::Utc;
use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

pub struct StatusRefresher {
    frequencies: Vec<StatusRefreshSettings>,
    status_service: Arc<dyn StatusService>,
}

impl StatusRefresher {
    pub fn new(container: Arc<Container>) -> Self {
        let mut frequencies = container.settings.application.status_refresh.clone();
        frequencies.sort_by_key(|f| f.max_age);

        Self {
            frequencies,
            status_service: container.status_service.clone(),
        }
    }

    pub async fn refresh_statuses(&self) -> Result<(), Box<dyn Error>> {
        for frequency in &self.frequencies {
            let since = Utc::now() - frequency.max_age.deref().clone();
            let fresh_since = Utc::now() - frequency.frequency.deref().clone();
            log::info!(
                "Refreshing statuses with age={:?} and frequency={:?} - since={:?} fresh_since={:?}",
                frequency.max_age.deref(),
                frequency.frequency.deref(),
                since,
                fresh_since
            );
            let status_ids: Vec<String> = self
                .status_service
                .list_stale_statuses(since, fresh_since)
                .await?;

            log::debug!("Found {} stale statuses", status_ids.len());

            let mut statuses = vec![];
            for (i, chunk) in status_ids.chunks(10).enumerate() {
                log::debug!("Refreshing chunk {}/{}", i + 1, status_ids.len() / 10 + 1);
                statuses.extend(self.status_service.fetch_statuses(chunk).await?);
                sleep(Duration::from_secs(5)).await;
            }
            self.status_service.persist_statuses(&statuses).await?;
            log::info!("Refreshed {} statuses", statuses.len());
        }
        Ok(())
    }
}

#[async_trait]
impl Worker for StatusRefresher {
    async fn run(&self, cancellation_token: CancellationToken) {
        let minimum_frequency_o = self.frequencies.iter().map(|f| f.frequency.deref()).min();
        let minimum_frequency: Duration = match minimum_frequency_o {
            Some(v) => v.clone(),
            None => {
                log::warn!(
                    "no status refresh frequency specified, not starting the status refresher"
                );
                return;
            }
        };

        log::info!("starting timeline updater worker");
        match self.refresh_statuses().await {
            Ok(_) => {}
            Err(e) => {
                log::error!("error while refreshing statuses: {}", e);
            }
        };
        loop {
            tokio::select! {
                _ = sleep(minimum_frequency) => {
                    match self.refresh_statuses().await {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("error while refreshing statuses: {}", e);
                        }
                    };
                    continue;
                }

                _ = cancellation_token.cancelled() => {
                    log::info!("gracefully shutting down the status refresher");
                    break;
                }
            }
        }
    }
}
