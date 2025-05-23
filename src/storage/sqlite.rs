use crate::storage::traits::{DataAccessLayer, StatusTagsCollection, StatusesIndexer};
use megalodon::entities::Status;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Row, ToSql, params};
use std::error::Error;

#[derive(Clone)]
pub struct SqliteDal(Pool<SqliteConnectionManager>);

impl SqliteDal {
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
}

impl SqliteDal {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let manager = SqliteConnectionManager::file("data/db.sqlite3");
        let pool = Pool::new(manager).expect("unable to create db pool");

        {
            let conn = pool.get()?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
            conn.pragma_update(None, "journal_mode", "MEMORY")?;
        }

        Self::create_sqlite_tables(&pool)?;
        Ok(Self(pool))
    }
}

impl StatusesIndexer for SqliteDal {
    fn insert_statuses(&self, statuses: Vec<&Status>) -> Result<(), Box<dyn Error>> {
        let mut conn = self.0.get()?;
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
}

impl StatusTagsCollection for SqliteDal {
    fn popular_tags(
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

impl DataAccessLayer for SqliteDal {}
