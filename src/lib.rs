//! Toy payments engine.
//!
//! Layers:
//! `domain` (pure business logic) ← `io` (CSV streaming and models).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod domain;
pub mod error;
pub mod io;

// Re-exports for convenience and backwards-compatibility
pub use domain::{account, amount, engine};
pub use io::{model, processor};
