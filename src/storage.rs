use chrono::{DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{File, exists};
use std::io::{Read, Write};
use std::sync::Mutex;

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
}

impl Storage {
    pub fn new() -> Result<Storage, Box<dyn Error>> {
        let config_exists = exists("data/storage.toml")?;
        if !config_exists {
            return Ok(Storage {
                hashtags: Mutex::new(HashtagTable::new()),
            });
        }

        let mut file = File::open("data/storage.toml")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let storage = toml::from_str(&contents)?;
        Ok(storage)
    }

    pub fn persist(&self) -> Result<(), Box<dyn Error>> {
        let str = toml::to_string(&self)?;
        let mut file = File::create("data/storage.toml")?;
        file.write_all(str.as_bytes())?;
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

    pub fn suggest_hashtag(&self, key: &str) -> () {
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
        self.persist().unwrap()
    }
}
