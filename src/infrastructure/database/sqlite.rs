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
