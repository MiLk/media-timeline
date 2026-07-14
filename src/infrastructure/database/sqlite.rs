use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::error::Error;
use std::ops::DerefMut;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

pub type Connection = Pool<SqliteConnectionManager>;

fn create_sqlite_tables(pool: &Connection) -> Result<(), Box<dyn Error>> {
    let mut conn = pool.get()?;
    embedded::migrations::runner()
        .run(conn.deref_mut())
        .unwrap();
    Ok(())
}

/// Apply the pragmas needed for concurrent access. WAL lets readers run
/// alongside the single writer, and `busy_timeout` makes a connection wait for
/// a lock instead of returning `SQLITE_BUSY` ("database is locked"). Applied via
/// the pool's init hook so every pooled connection gets them, not just the first.
fn configure_connection(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "busy_timeout", "5000")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(())
}

pub fn new() -> Result<Connection, Box<dyn Error>> {
    let manager = SqliteConnectionManager::file("data/db.sqlite3")
        .with_init(|conn| configure_connection(conn));
    let pool = Pool::new(manager).expect("unable to create db pool");

    create_sqlite_tables(&pool)?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn configure_connection_enables_concurrency_pragmas() {
        let path = std::env::temp_dir().join(format!("mt-sqlite-test-{}.db", std::process::id()));
        let conn = Connection::open(&path).unwrap();

        configure_connection(&conn).unwrap();

        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode, "wal");
        let busy_timeout: i64 = conn
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .unwrap();
        assert_eq!(busy_timeout, 5000);

        drop(conn);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
    }
}
