use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ApplicationSettings {
    timeline_update_frequency_seconds: u64,
}

impl ApplicationSettings {
    pub(crate) fn timeline_update_frequency(&self) -> Duration {
        Duration::from_secs(self.timeline_update_frequency_seconds)
    }
}
