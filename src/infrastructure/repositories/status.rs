use crate::domain::repositories::status::{RecentStatusRepository, StatusIndexRepository};
use crate::infrastructure::database::sqlite;
use async_trait::async_trait;
use megalodon::entities::Status;
use rusqlite::fallible_iterator::FallibleIterator;
use rusqlite::{OptionalExtension, Row, ToSql, params};
use std::error::Error;
use std::sync::Arc;

pub struct RecentStatusSqliteRepository {
    pool: Arc<sqlite::Connection>,
}

impl RecentStatusSqliteRepository {
    pub fn new(pool: Arc<sqlite::Connection>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RecentStatusRepository for RecentStatusSqliteRepository {
    fn get_recent_status_id(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        let conn = self.pool.get()?;
        let mut stmt =
            conn.prepare_cached("SELECT status_id FROM recent_statuses WHERE tag = ?1")?;
        let result = stmt.query_row(params![key], |row| row.get(0)).optional()?;
        Ok(result)
    }

    fn set_recent_status_id(&self, key: &String, value: &String) -> Result<(), Box<dyn Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(
            "INSERT OR REPLACE INTO recent_statuses(tag, status_id) VALUES (?1, ?2);",
        )?;
        stmt.execute(params![key, value])?;
        Ok(())
    }
}

pub struct StatusSqliteRepository {
    pool: Arc<sqlite::Connection>,
}

impl StatusSqliteRepository {
    pub fn new(pool: Arc<sqlite::Connection>) -> Self {
        Self { pool }
    }
}

impl StatusIndexRepository for StatusSqliteRepository {
    fn insert_statuses(&self, statuses: Vec<&Status>) -> Result<(), Box<dyn Error>> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached("INSERT OR REPLACE INTO statuses (id, created_at, account_id, account_acct) VALUES (?1, ?2, ?3, ?4)")?;
            let mut tag_stmt = tx.prepare_cached(
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
        }
        tx.commit()?;
        Ok(())
    }

    fn search_statuses(
        &self,
        hashtags_o: Option<&Vec<String>>,
        limit: u16,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let hashtags_clause: String = match hashtags_o {
            Some(hashtags) => {
                let n = hashtags.len();
                let mut s = "?,".repeat(n);
                s.pop();
                format!("WHERE lower(st.name) IN ({})", s)
            }
            None => String::new(),
        };

        let conn = self.pool.get()?;
        let sql = format!(
            "SELECT DISTINCT(s.id)
            FROM statuses s
            LEFT JOIN status_tags st ON st.status_id = s.id
            {}
            ORDER BY s.created_at DESC
            LIMIT :limit;",
            hashtags_clause
        );
        let mut stmt = conn.prepare(&sql)?;

        // use raw_bind_parameter because we mix parameters of different types
        // and dynamic number of parameters
        if let Some(hashtags) = hashtags_o {
            for (i, tag) in hashtags.iter().enumerate() {
                stmt.raw_bind_parameter(i + 1, tag.to_lowercase())?;
            }
        }
        stmt.raw_bind_parameter(c":limit", limit)?;

        let statuses: rusqlite::Result<Vec<String>> =
            stmt.raw_query().map(|row| row.get(0)).collect();
        Ok(statuses?)
    }

    fn popular_tags(
        &self,
        duration_days: &u16,
        limit: &u16,
    ) -> Result<Vec<(String, u32)>, Box<dyn Error>> {
        let conn = self.pool.get()?;
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
