use duration::DurationValue;
use serde::Deserialize;

pub mod duration;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StatusRefreshSettings {
    pub max_age: DurationValue,
    pub frequency: DurationValue,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ApplicationSettings {
    pub timeline_update_frequency: DurationValue,
    pub timeline_statuses_count: u16,
    pub status_refresh: Vec<StatusRefreshSettings>,
}
