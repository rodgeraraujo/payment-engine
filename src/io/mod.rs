//! I/O adapters: CSV deserialization and serialization.

pub mod model;
pub mod processor;

pub use model::{RawRecord, Transaction};
pub use processor::{process, write_accounts, Stats};
