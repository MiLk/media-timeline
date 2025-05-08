use chrono::{DateTime, Utc};
use glob::glob;
use log::debug;
use megalodon::entities::Status;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::exists;
use std::sync::Mutex;
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
pub struct Storage {
    hashtags: Mutex<HashtagTable>,
    recent_status_ids: Mutex<HashMap<String, String>>,
}

impl Storage {
    pub async fn new() -> Result<Storage, Box<dyn Error>> {
        let config_exists = exists("data/storage.toml")?;
        if !config_exists {
            return Ok(Storage {
                hashtags: Mutex::new(HashtagTable::new()),
                recent_status_ids: Mutex::new(HashMap::new()),
            });
        }

        let mut file = File::open("data/storage.toml").await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        let storage = toml::from_str(&contents)?;
        Ok(storage)
    }

    pub async fn persist(&self) -> Result<(), Box<dyn Error>> {
        let str = toml::to_string(&self)?;
        let mut file = File::create("data/storage.toml").await?;
        file.write_all(str.as_bytes()).await?;
        Ok(())
    }

    pub fn list_hashtags(&self) -> Vec<HashtagKey> {
        let data = self.hashtags.lock().unwrap();
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

    pub async fn suggest_hashtag(&self, key: &str) -> () {
        if key.is_empty() {
            return;
        }

        {
            let mut data = self.hashtags.lock().unwrap();
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

    pub fn get_recent_status_id(&self, key: &str) -> Option<String> {
        self.recent_status_ids
            .lock()
            .unwrap()
            .get(key)
            .map(|s| s.clone())
    }

    pub fn set_recent_status_id(&self, key: &String, value: &String) -> Option<String> {
        self.recent_status_ids
            .lock()
            .unwrap()
            .insert(key.clone(), value.clone())
    }

    pub async fn persist_statuses(&self, statuses: &Vec<Status>) -> Result<(), Box<dyn Error>> {
        async fn write_status(status: Status) -> std::io::Result<()> {
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
            create_dir_all(&dir).await?;
            let mut file = File::create(format!("{}/{}.json", dir, &status_id)).await?;
            file.write_all(serde_json::to_string(&status)?.as_bytes())
                .await
        }

        let mut tasks = Vec::with_capacity(statuses.len());
        for status in statuses.iter() {
            tasks.push(tokio::spawn(write_status(status.clone())))
        }
        for task in tasks {
            task.await??;
        }
        self.persist().await?; // Write the recent IDs to disk
        Ok(())
    }

    pub async fn retrieve_statuses(
        &self,
        hashtags: &Vec<String>,
    ) -> Result<Vec<Status>, Box<dyn Error>> {
        let set: HashSet<String> = hashtags.iter().map(|s| s.to_lowercase()).collect();
        let mut statuses = Vec::new();
        for entry in glob("data/statuses/**/*.json")? {
            let path = entry?;
            let mut file = File::open(path).await?;
            let mut content = String::new();
            file.read_to_string(&mut content).await?;
            let decoded: Status = serde_json::from_str(content.as_str())?;
            if decoded
                .tags
                .iter()
                .any(|t| set.contains(t.name.to_lowercase().as_str()))
            {
                statuses.push(decoded);
            }
        }
        debug!("{} statuses read from storage", statuses.len());
        Ok(statuses)
    }
}
