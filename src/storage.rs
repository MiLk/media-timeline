use chrono::{DateTime, Utc};
use glob::glob;
use log::debug;
use megalodon::entities::Status;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::types::ToSql;
use rusqlite::{Row, params};
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

#[derive(Clone)]
struct DbWrapper(Pool<SqliteConnectionManager>);

impl DbWrapper {
    fn new() -> Result<Self, Box<dyn Error>> {
        let manager = SqliteConnectionManager::file("data/db.sqlite3");
        let pool = Pool::new(manager).expect("unable to create db pool");
        Self::create_sqlite_tables(&pool)?;
        Ok(Self(pool))
    }

    fn create_sqlite_tables(pool: &Pool<SqliteConnectionManager>) -> Result<(), Box<dyn Error>> {
        let conn = pool.get()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS statuses (
                id   TEXT NOT NULL PRIMARY KEY,
                created_at TEXT NOT NULL,
                account_id TEXT NOT NULL,
                account_acct TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS status_tags (
                status_id   TEXT NOT NULL,
                name   TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS status_tags_idx ON status_tags (status_id, name);",
        )?;
        Ok(())
    }

    pub fn insert_statuses<'a>(
        &self,
        statuses: impl IntoIterator<Item = &'a Status>,
    ) -> Result<(), Box<dyn Error>> {
        let conn = self.0.get()?;
        let mut stmt = conn.prepare_cached("INSERT OR REPLACE INTO statuses (id, created_at, account_id, account_acct) VALUES (?1, ?2, ?3, ?4)")?;
        let mut tag_stmt = conn.prepare_cached(
            "INSERT OR REPLACE INTO status_tags (status_id, name) VALUES (?1, ?2)",
        )?;
        for status in statuses {
            let created_at = ToSql::to_sql(&status.created_at)?;
            stmt.execute(params![
                &status.id,
                &created_at,
                &status.account.id,
                &status.account.acct
            ])?;

            for tag in &status.tags {
                tag_stmt.execute(params![&status.id, &tag.name])?;
            }
        }
        Ok(())
    }

    pub fn popular_tags(
        &self,
        duration_days: &u16,
        limit: &u16,
    ) -> Result<Vec<(String, u32)>, Box<dyn Error>> {
        let conn = self.0.get()?;
        let mut stmt = conn.prepare_cached(
            "SELECT name, COUNT(*)
            FROM status_tags st
            LEFT JOIN statuses s ON st.status_id = s.id
            WHERE s.created_at >= datetime('now', ?1)
            GROUP BY name
            ORDER BY 2 DESC
            LIMIT ?2;",
        )?;

        fn read_row(row: &Row) -> rusqlite::Result<(String, u32)> {
            Ok((row.get(0)?, row.get(1)?))
        }

        let results: Result<Vec<(String, u32)>, _> = stmt
            .query_map(
                params![format!("-{} days", &duration_days), &limit],
                read_row,
            )?
            .collect();
        Ok(results?)
    }
}

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
    db: DbWrapper,
    data: StorageData,
}

impl Storage {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let data = if exists("data/storage.toml")? {
            StorageData::from_disk().await?
        } else {
            StorageData::new()
        };

        Ok(Storage {
            db: DbWrapper::new()?,
            data,
        })
    }

    pub async fn persist(&self) -> Result<(), Box<dyn Error>> {
        let str = toml::to_string(&self.data)?;
        let mut file = File::create("data/storage.toml").await?;
        file.write_all(str.as_bytes()).await?;
        Ok(())
    }

    pub fn list_hashtags(&self) -> Vec<HashtagKey> {
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

    pub async fn suggest_hashtag(&self, key: &str) -> () {
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

    pub fn get_recent_status_id(&self, key: &str) -> Option<String> {
        self.data
            .recent_status_ids
            .lock()
            .unwrap()
            .get(key)
            .map(|s| s.clone())
    }

    pub fn set_recent_status_id(&self, key: &String, value: &String) -> Option<String> {
        self.data
            .recent_status_ids
            .lock()
            .unwrap()
            .insert(key.clone(), value.clone())
    }

    pub async fn persist_statuses(&self, statuses: &Vec<Status>) -> Result<(), Box<dyn Error>> {
        async fn write_status(status: Status, db: DbWrapper) -> () {
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

            db.insert_statuses(std::iter::once(&status))
                .expect(format!("failed to insert status {}", &status_id).as_str());
        }

        let mut tasks = Vec::with_capacity(statuses.len());
        for status in statuses.iter() {
            tasks.push(tokio::spawn(write_status(status.clone(), self.db.clone())))
        }
        for task in tasks {
            task.await?;
        }

        self.persist().await?; // Write the recent IDs to disk
        Ok(())
    }

    pub async fn retrieve_statuses(
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

    pub async fn rebuild_index_statuses(&self) -> Result<(), Box<dyn Error>> {
        debug!("Rebuilding index statuses");

        let statuses = self.retrieve_statuses(None).await?;
        self.db.insert_statuses(statuses.iter())?;
        Ok(())
    }

    pub fn popular_tags(
        &self,
        periods: Vec<u16>,
        limit: u16,
    ) -> Result<HashMap<u16, Vec<(String, u32)>>, Box<dyn Error>> {
        periods
            .iter()
            .map(|&period| Ok((period, self.db.popular_tags(&period, &limit)?)))
            .collect()
    }
}
