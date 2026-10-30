//! Error types. Every failure mode is a typed variant so callers (and tests)
//! can match on it; nothing in the engine panics on bad input.

use thiserror::Error;

/// Problems turning text into a valid monetary [`Amount`](crate::amount::Amount).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AmountError {
    /// String could not be parsed as a decimal number.
    #[error("invalid decimal number: {0:?}")]
    Invalid(String),
    /// Value is zero or negative.
    #[error("amount must be strictly positive")]
    NotPositive,
    /// Value has more than 4 fractional decimal digits.
    #[error("amount has more than 4 decimal places")]
    TooPrecise,
}

/// Problems turning a raw CSV row into a typed [`Transaction`](crate::model::Transaction).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    /// The transaction type is not recognized.
    #[error("unknown transaction type: {0:?}")]
    UnknownType(String),
    /// A deposit or withdrawal row was missing an amount.
    #[error("{0} requires an amount")]
    MissingAmount(&'static str),
    /// The amount value could not be parsed.
    #[error(transparent)]
    Amount(#[from] AmountError),
}

/// Business-rule rejections. These are expected in real traffic (partner
/// errors, fraud attempts) and are logged and skipped, never fatal.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EngineError {
    /// The client does not have enough available balance for the withdrawal.
    #[error("client {client}: insufficient available funds for tx {tx}")]
    InsufficientFunds {
        /// Client identifier.
        client: u16,
        /// Transaction identifier.
        tx: u32,
    },
    /// The account has been frozen following a chargeback.
    #[error("client {0}: account is locked")]
    AccountLocked(u16),
    /// A deposit was submitted with an already-used transaction identifier.
    #[error("tx {0}: duplicate transaction id")]
    DuplicateTx(u32),
    /// The referenced deposit was not found in the transaction history.
    #[error("tx {0}: referenced transaction not found")]
    UnknownTx(u32),
    /// The referenced deposit was registered to a different client.
    #[error("tx {tx}: belongs to a different client than {client}")]
    ClientMismatch {
        /// Referenced transaction identifier.
        tx: u32,
        /// Disputing client identifier.
        client: u16,
    },
    /// The referenced deposit is not in a valid state for this action.
    #[error("tx {tx}: cannot {action} a transaction in state {state}")]
    InvalidState {
        /// Transaction identifier.
        tx: u32,
        /// Action attempted.
        action: &'static str,
        /// Current lifecycle state.
        state: &'static str,
    },
    /// An arithmetic overflow occurred while updating balances for this client.
    #[error("arithmetic overflow on client {0}")]
    Overflow(u16),
}
