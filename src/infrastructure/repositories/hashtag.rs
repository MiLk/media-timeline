use crate::domain::repositories::hashtag::SubscribedHashtagRepository;
use crate::infrastructure::database::sqlite;
use async_trait::async_trait;
use rusqlite::{OptionalExtension, params};
use std::error::Error;
use std::sync::Arc;

pub struct SubscribedHashtagSqliteRepository {
    pool: Arc<sqlite::Connection>,
}

impl SubscribedHashtagSqliteRepository {
    pub fn new(pool: Arc<sqlite::Connection>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubscribedHashtagRepository for SubscribedHashtagSqliteRepository {
    fn increment_vote(&self, key: &str) -> Result<(), Box<dyn Error>> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        {
            let mut select_stmt =
                tx.prepare_cached("SELECT COUNT(*) FROM subscribed_hashtags WHERE name = ?1")?;
            let count = select_stmt.query_row(params![key], |row| row.get(0))?;

            match count {
                0 => {
                    let mut stmt = tx.prepare_cached(
                        "INSERT INTO subscribed_hashtags (name, votes) VALUES(?1, ?2);",
                    )?;
                    stmt.execute(params![key, 1])?;
                }
                _ => {
                    let mut stmt = tx.prepare_cached(
                        "UPDATE subscribed_hashtags SET votes = votes + 1 WHERE name = ?1;",
                    )?;
                    stmt.execute(params![key])?;
                }
            };
        }
        tx.commit()?;
        Ok(())
    }

    fn list(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(
            "SELECT name FROM subscribed_hashtags WHERE approved = 1 ORDER BY name",
        )?;
        let result = stmt.query_map((), |row| row.get(0)).optional()?;

        let mut results: Vec<String> = Vec::new();
        match result {
            Some(rows) => {
                for row in rows {
                    results.push(row?);
                }
            }
            _ => {}
        };
        Ok(results)
    }
}
