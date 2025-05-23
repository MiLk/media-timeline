pub mod sqlite;
pub mod traits;

use crate::storage::traits::{
    DataAccessLayer, RecentStatusesService, StatusesIndexer, StatusesService,
    SubscribedHashtagsService,
};
use actix_web::web;
use chrono::{DateTime, Utc};
use glob::glob;
use log::debug;
use megalodon::entities::Status;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::exists;
use std::sync::{Arc, Mutex};
use tokio::fs::{File, create_dir_all};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

type HashtagKey = String;

#[derive(Serialize, Deserialize)]
pub struct HashtagAttributes {
    approved: bool,
    votes: u16,
    created_at: DateTime<Utc>,
}

impl HashtagAttributes {
    pub fn new_suggestion() -> Self {
        Self {
            approved: false,
            votes: 1,
            created_at: Utc::now(),
        }
    }
}

type HashtagTable = HashMap<HashtagKey, HashtagAttributes>;

#[derive(Serialize, Deserialize)]
pub struct StorageData {
    hashtags: Mutex<HashtagTable>,
    recent_status_ids: Mutex<HashMap<String, String>>,
}

impl StorageData {
    fn new() -> Self {
        Self {
            hashtags: Mutex::new(HashtagTable::new()),
            recent_status_ids: Mutex::new(HashMap::new()),
        }
    }

    async fn from_disk() -> Result<Self, Box<dyn Error>> {
        let mut file = File::open("data/storage.toml").await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        let parsed = toml::from_str(&contents)?;
        Ok(parsed)
    }
}

pub struct Storage {
    dal: Arc<dyn DataAccessLayer>,
    data: StorageData,
}

impl Storage {
    pub async fn new<T: DataAccessLayer>(dal: T) -> Result<Self, Box<dyn Error>> {
        let data = if exists("data/storage.toml")? {
            StorageData::from_disk().await?
        } else {
            StorageData::new()
        };

        let storage = Self {
            dal: Arc::new(dal),
            data,
        };
        storage.rebuild_index_statuses().await?;

        Ok(storage)
    }

    /// Rebuild indexes in case it's not up to date
    /// This will read all the cached posts on the disk to rebuild the index database
    async fn rebuild_index_statuses(&self) -> Result<(), Box<dyn Error>> {
        debug!("Rebuilding index statuses");
        let statuses = self.retrieve_statuses(None).await?;

        let dal = self.dal.clone();
        web::block(move || {
            dal.insert_statuses(statuses.iter().collect())
                .expect("unable to rebuild the index for statuses")
        })
        .await?;
        Ok(())
    }

    async fn persist(&self) -> Result<(), Box<dyn Error>> {
        let str = toml::to_string(&self.data)?;
        let mut file = File::create("data/storage.toml").await?;
        file.write_all(str.as_bytes()).await?;
        Ok(())
    }
}

impl StatusesService for Storage {
    async fn persist_statuses(&self, statuses: &Vec<Status>) -> Result<(), Box<dyn Error>> {
        async fn write_status(status: Status, db: Arc<dyn StatusesIndexer + Sync + Send>) -> () {
            let status_id = status.id.clone();
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
            let dir = format!("data/statuses/{}/{}", dir1, dir2);
            create_dir_all(&dir)
                .await
                .expect(format!("failed to create dir {}", dir).as_str());
            let filepath = format!("{}/{}.json", dir, &status_id);
            let mut file = File::create(&filepath)
                .await
                .expect(format!("failed to create file {}", &filepath).as_str());
            file.write_all(
                serde_json::to_string(&status)
                    .expect(format!("failed to serialize status {}", &status_id).as_str())
                    .as_bytes(),
            )
            .await
            .expect(format!("failed to write to file {}", &filepath).as_str());

            db.insert_statuses(vec![&status])
                .expect(format!("failed to insert status {}", &status_id).as_str());
        }

        let mut tasks = Vec::with_capacity(statuses.len());
        for status in statuses.iter() {
            tasks.push(tokio::spawn(write_status(status.clone(), self.dal.clone())))
        }
        for task in tasks {
            task.await?;
        }

        self.persist().await?; // Write the recent IDs to disk
        Ok(())
    }

    async fn retrieve_statuses(
        &self,
        hashtags: Option<&Vec<String>>,
    ) -> Result<Vec<Status>, Box<dyn Error>> {
        let set_o: Option<HashSet<String>> =
            hashtags.map(|hts| hts.iter().map(|s| s.to_lowercase()).collect());
        let mut statuses = Vec::new();
        for entry in glob("data/statuses/**/*.json")? {
            let path = entry?;
            let mut file = File::open(path).await?;
            let mut content = String::new();
            file.read_to_string(&mut content).await?;
            let decoded: Status = serde_json::from_str(content.as_str())?;
            if match &set_o {
                Some(set) => decoded
                    .tags
                    .iter()
                    .any(|t| set.contains(t.name.to_lowercase().as_str())),
                None => true,
            } {
                statuses.push(decoded);
            }
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
            .map(|&period| Ok((period, self.dal.popular_tags(&period, &limit)?)))
            .collect()
    }
}

impl RecentStatusesService for Storage {
    fn get_recent_status_id(&self, key: &str) -> Option<String> {
        self.data
            .recent_status_ids
            .lock()
            .unwrap()
            .get(key)
            .map(|s| s.clone())
    }

    fn set_recent_status_id(&self, key: &String, value: &String) -> Option<String> {
        self.data
            .recent_status_ids
            .lock()
            .unwrap()
            .insert(key.clone(), value.clone())
    }
}

impl SubscribedHashtagsService for Storage {
    fn list_hashtags(&self) -> Vec<HashtagKey> {
        let data = self.data.hashtags.lock().unwrap();
        let mut list: Vec<HashtagKey> = data
            .iter()
            .filter_map(|(key, attrs)| match attrs.approved {
                true => Some(key.clone()),
                false => None,
            })
            .collect();
        list.sort();
        list
    }

    async fn suggest_hashtag(&self, key: &str) -> () {
        if key.is_empty() {
            return;
        }

        {
            let mut data = self.data.hashtags.lock().unwrap();
            match data.get_mut(key) {
                Some(value) => {
                    value.votes += 1;
                    debug!(
                        "Suggested hashtag received a new vote: {} => {}",
                        key, value.votes
                    );
                }
                None => {
                    debug!("New hashtag suggested: {}", key);
                    data.insert(String::from(key), HashtagAttributes::new_suggestion());
                }
            }
        }
        self.persist().await.unwrap()
    }
}
