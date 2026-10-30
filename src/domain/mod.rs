//! Domain logic: monetary amounts, client accounts, and the stateful engine.
//!
//! Modules in `domain` have zero dependencies on I/O.

pub mod account;
pub mod amount;
pub mod engine;

pub use account::Account;
pub use amount::Amount;
pub use engine::{Engine, Transaction};
