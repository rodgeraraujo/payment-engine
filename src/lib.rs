//! Toy payments engine.
//!
//! Layers (each depends only on those above it):
//! `amount` → `account` → `engine` (pure logic) and `model` → `processor` (CSV I/O).

pub mod account;
pub mod amount;
pub mod engine;
pub mod error;
pub mod model;
pub mod processor;
