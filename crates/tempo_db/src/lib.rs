mod schema;

use rusqlite::Connection;

// TODO does this really need to be a dashmap?

/// Holds all of Tempo's open SQLite connections
pub struct DbConns(dashmap::DashMap<std::path::PathBuf, std::sync::Mutex<Connection>>);
static DB_CONNS: std::sync::LazyLock<DbConns> =
    std::sync::LazyLock::new(|| DbConns(dashmap::DashMap::new()));

/// A connection to Tempo's SQLite database.
pub struct Db {}

/// Gets a connection to the given database. Blocking.
pub fn get<P>(path: P) -> anyhow::Result<Db>
where
    P: AsRef<std::path::Path>,
{
    todo!()
}

pub async fn get_async<P>(path: P) -> anyhow::Result<Db>
where
    P: AsRef<std::path::Path>,
{
    todo!()
}
