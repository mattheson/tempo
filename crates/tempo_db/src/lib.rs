mod connection;
mod db;
mod dbs;
mod error;
mod notify;
mod schema;

pub use dbs::Dbs;
// pub use db::Db;
pub use connection::*;
pub use db::RootDb;
pub use error::Error;
pub use notify::DbNotifier;

pub type Result<T> = std::result::Result<T, Error>;
