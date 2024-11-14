use crate::db::TempoDb;

pub mod fs;

/// Providers handle perisistance and synchronization of data.
pub trait TempoProvider {
    fn new(db: TempoDb) -> Self;
}