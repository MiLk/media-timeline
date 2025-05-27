use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HashtagAttributes {
    pub approved: bool,
    pub votes: u16,
    pub created_at: DateTime<Utc>,
}

impl Default for HashtagAttributes {
    fn default() -> Self {
        Self {
            approved: false,
            votes: 1,
            created_at: Utc::now(),
        }
    }
}
