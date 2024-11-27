mod schema;
mod dbs;
mod db;

pub use dbs::Dbs;
pub use db::Db;

pub type DbConnection = std::sync::Arc<tokio::sync::Mutex<rusqlite::Connection>>;
