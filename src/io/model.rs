//! Input model: a loosely-typed CSV row ([`RawRecord`]) and the validated
//! domain type ([`Transaction`]) built from it. Parsing happens once, at the
//! edge; the engine only ever sees valid, typed transactions.

use crate::domain::amount::Amount;
use crate::error::ParseError;
use serde::Deserialize;

pub use crate::domain::engine::Transaction;

/// One CSV row exactly as it appears in the file. `amount` is optional because
/// dispute/resolve/chargeback rows omit it (possibly without a trailing comma).
#[derive(Debug, Deserialize)]
pub struct RawRecord {
    /// Transaction type name (e.g. `deposit`, `withdrawal`, `dispute`, `resolve`, `chargeback`).
    #[serde(rename = "type")]
    pub kind: String,
    /// Client identifier.
    pub client: u16,
    /// Transaction identifier.
    pub tx: u32,
    /// Optional decimal amount string.
    pub amount: Option<String>,
}

impl TryFrom<RawRecord> for Transaction {
    type Error = ParseError;

    fn try_from(r: RawRecord) -> Result<Self, ParseError> {
        let (client, tx) = (r.client, r.tx);
        let amount = |name: &'static str| -> Result<Amount, ParseError> {
            let raw = r.amount.as_deref().filter(|s| !s.trim().is_empty());
            Ok(Amount::parse_positive(
                raw.ok_or(ParseError::MissingAmount(name))?,
            )?)
        };
        match r.kind.trim().to_ascii_lowercase().as_str() {
            "deposit" => Ok(Transaction::Deposit {
                client,
                tx,
                amount: amount("deposit")?,
            }),
            "withdrawal" => Ok(Transaction::Withdrawal {
                client,
                tx,
                amount: amount("withdrawal")?,
            }),
            "dispute" => Ok(Transaction::Dispute { client, tx }),
            "resolve" => Ok(Transaction::Resolve { client, tx }),
            "chargeback" => Ok(Transaction::Chargeback { client, tx }),
            other => Err(ParseError::UnknownType(other.to_owned())),
        }
    }
}
