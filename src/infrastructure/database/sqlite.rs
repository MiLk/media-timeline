use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::error::Error;

pub type Connection = Pool<SqliteConnectionManager>;

fn create_sqlite_tables(pool: &Connection) -> Result<(), Box<dyn Error>> {
    let conn = pool.get()?;
    conn.execute_batch(
        "
            CREATE TABLE IF NOT EXISTS subscribed_hashtags(
              name TEXT NOT NULL PRIMARY KEY,
                approved INTEGER NOT NULL DEFAULT 0,
                votes INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (DATETIME('now'))
            );
            CREATE TABLE IF NOT EXISTS recent_statuses(
                tag TEXT NOT NULL PRIMARY KEY,
                status_id TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS statuses(
                id TEXT NOT NULL PRIMARY KEY,
                created_at TEXT NOT NULL,
                account_id TEXT NOT NULL,
                account_acct TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS status_tags(
                status_id TEXT NOT NULL,
                name TEXT NOT NULL);
            CREATE INDEX IF NOT EXISTS status_tags_idx ON status_tags (status_id, name);
            CREATE TABLE IF NOT EXISTS status_refreshes(
                id TEXT NOT NULL PRIMARY KEY,
                refreshed_at TEXT NOT NULL);
            ",
    )?;
    Ok(())
}

pub fn new() -> Result<Connection, Box<dyn Error>> {
    let manager = SqliteConnectionManager::file("data/db.sqlite3");
    let pool = Pool::new(manager).expect("unable to create db pool");

    {
        let conn = pool.get()?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "journal_mode", "MEMORY")?;
    }

    create_sqlite_tables(&pool)?;
    Ok(pool)
}
