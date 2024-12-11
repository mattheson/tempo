mod schema;
mod dbs;
mod db;
mod notify;

pub use dbs::Dbs;
pub use db::Db;
pub use notify::DbNotifier;

pub type DbConnection = std::sync::Arc<tokio::sync::Mutex<rusqlite::Connection>>;
