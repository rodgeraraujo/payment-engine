//! Error types. Every failure mode is a typed variant so callers (and tests)
//! can match on it; nothing in the engine panics on bad input.

use thiserror::Error;

/// Problems turning text into a valid monetary [`Amount`](crate::amount::Amount).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AmountError {
    #[error("invalid decimal number: {0:?}")]
    Invalid(String),
    #[error("amount must be strictly positive")]
    NotPositive,
    #[error("amount has more than 4 decimal places")]
    TooPrecise,
}

/// Problems turning a raw CSV row into a typed [`Transaction`](crate::model::Transaction).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("unknown transaction type: {0:?}")]
    UnknownType(String),
    #[error("{0} requires an amount")]
    MissingAmount(&'static str),
    #[error(transparent)]
    Amount(#[from] AmountError),
}

/// Business-rule rejections. These are expected in real traffic (partner
/// errors, fraud attempts) and are logged and skipped, never fatal.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EngineError {
    #[error("client {client}: insufficient available funds for tx {tx}")]
    InsufficientFunds { client: u16, tx: u32 },
    #[error("client {0}: account is locked")]
    AccountLocked(u16),
    #[error("tx {0}: duplicate transaction id")]
    DuplicateTx(u32),
    #[error("tx {0}: referenced transaction not found")]
    UnknownTx(u32),
    #[error("tx {tx}: belongs to a different client than {client}")]
    ClientMismatch { tx: u32, client: u16 },
    #[error("tx {tx}: cannot {action} a transaction in state {state}")]
    InvalidState {
        tx: u32,
        action: &'static str,
        state: &'static str,
    },
    #[error("arithmetic overflow on client {0}")]
    Overflow(u16),
}
