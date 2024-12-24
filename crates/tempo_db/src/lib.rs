mod schema;
mod dbs;
mod db;
mod notify;
mod connection;
mod error;

pub use dbs::Dbs;
pub use db::Db;
pub use error::Error;
pub use notify::DbNotifier;
pub use connection::*;

pub type RawConnection = std::sync::Arc<Connection>;
pub type Result<T> = std::result::Result<T, Error>;