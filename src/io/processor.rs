//! I/O layer: streams CSV in, applies each row to the [`Engine`], writes CSV out.
//! Generic over `Read`/`Write` so it works for files, stdin, or (later) sockets,
//! and is trivially testable with in-memory buffers.

use crate::domain::engine::Engine;
use crate::domain::engine::Transaction;
use crate::error::{AmountError, EngineError, ParseError};
use crate::io::model::RawRecord;
use std::collections::BTreeMap;
use std::io::{Read, Write};

/// Counters returned for observability and tests.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Number of transactions successfully validated and applied.
    pub applied: u64,
    /// Number of rows skipped due to parse errors or rejected transactions.
    pub skipped: u64,
    /// Breakdown of skipped rows by category.
    pub skipped_by_reason: BTreeMap<&'static str, u64>,
}

impl Stats {
    fn record_skip(&mut self, reason: &'static str, line: usize, detail: &dyn std::fmt::Display) {
        self.skipped += 1;
        *self.skipped_by_reason.entry(reason).or_default() += 1;
        if self.skipped <= 10 {
            eprintln!("line {line}: skipped ({reason}): {detail}");
        } else if self.skipped == 11 {
            eprintln!("... further per-row error logs suppressed, summary will follow");
        }
    }
}

fn categorize_parse_error(err: &ParseError) -> &'static str {
    match err {
        ParseError::UnknownType(_) => "unknown transaction type",
        ParseError::MissingAmount(_) => "missing amount",
        ParseError::Amount(AmountError::Invalid(_)) => "invalid decimal",
        ParseError::Amount(AmountError::NotPositive) => "amount not positive",
        ParseError::Amount(AmountError::TooPrecise) => "amount has more than 4 decimal places",
    }
}

fn categorize_engine_error(err: &EngineError) -> &'static str {
    match err {
        EngineError::InsufficientFunds { .. } => "insufficient funds",
        EngineError::AccountLocked(_) => "account locked",
        EngineError::DuplicateTx(_) => "duplicate transaction id",
        EngineError::UnknownTx(_) => "unknown transaction id",
        EngineError::ClientMismatch { .. } => "client mismatch",
        EngineError::InvalidState { .. } => "invalid dispute state",
        EngineError::Overflow(_) => "arithmetic overflow",
    }
}

/// Streams `input` through `engine`, one record at a time (O(1) memory per row).
/// Malformed rows and rejected transactions are logged to stderr and skipped;
/// only I/O failures abort processing.
pub fn process<R: Read>(engine: &mut Engine, input: R) -> Result<Stats, csv::Error> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All) // "type, client, tx, amount" has spaces
        .flexible(true) // dispute/resolve/chargeback rows have no amount field
        .from_reader(input);

    let mut stats = Stats::default();
    for (index, row) in reader.deserialize::<RawRecord>().enumerate() {
        let line = index + 2; // +1 for header, +1 for 1-based numbering
        let raw = match row {
            Ok(raw) => raw,
            Err(e) if matches!(e.kind(), csv::ErrorKind::Io(_)) => return Err(e),
            Err(e) => {
                stats.record_skip("malformed CSV record", line, &e);
                continue;
            }
        };
        let transaction = match Transaction::try_from(raw) {
            Ok(tx) => tx,
            Err(e) => {
                stats.record_skip(categorize_parse_error(&e), line, &e);
                continue;
            }
        };
        match engine.apply(transaction) {
            Ok(()) => stats.applied += 1,
            Err(e) => {
                stats.record_skip(categorize_engine_error(&e), line, &e);
            }
        }
    }
    Ok(stats)
}

/// Writes final balances as CSV, ordered by client id.
pub fn write_accounts<W: Write>(engine: &Engine, output: W) -> Result<(), csv::Error> {
    let mut writer = csv::Writer::from_writer(output);
    writer.write_record(["client", "available", "held", "total", "locked"])?;
    for (id, account) in engine.accounts_sorted() {
        let total = account.total().ok_or_else(|| {
            csv::Error::from(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("arithmetic overflow computing total for client {id}"),
            ))
        })?;
        writer.write_record([
            id.to_string(),
            account.available().to_string(),
            account.held().to_string(),
            total.to_string(),
            account.is_locked().to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_works_with_in_memory_slice() {
        let mut engine = Engine::new();
        let input = b"type, client, tx, amount\n\
                      deposit, 1, 1, 10.0\n\
                      withdrawal, 1, 2, 3.5\n";
        let stats = process(&mut engine, &input[..]).expect("processing failed");
        assert_eq!(
            stats,
            Stats {
                applied: 2,
                skipped: 0,
                ..Default::default()
            }
        );

        let mut output = Vec::new();
        write_accounts(&engine, &mut output).expect("write failed");
        let csv_str = String::from_utf8(output).expect("utf8");
        assert_eq!(
            csv_str,
            "client,available,held,total,locked\n\
             1,6.5000,0.0000,6.5000,false\n"
        );
    }

    #[test]
    fn output_edge_cases_formatting() {
        let mut engine = Engine::new();
        // 1. Trailing zeros and large values: 1000000.5 -> 1000000.5000
        // 2. Negative available after dispute on spent funds
        let input = b"type, client, tx, amount\n\
                      deposit, 1, 1, 1000000.5\n\
                      withdrawal, 1, 2, 1000000.5\n\
                      dispute, 1, 1,\n\
                      withdrawal, 2, 3, 50.0\n"; // failed withdrawal only
        let stats = process(&mut engine, &input[..]).expect("processing failed");
        assert_eq!(stats.applied, 3);
        assert_eq!(stats.skipped, 1);
        assert_eq!(stats.skipped_by_reason.get("insufficient funds"), Some(&1));

        let mut output = Vec::new();
        write_accounts(&engine, &mut output).expect("write failed");
        let csv_str = String::from_utf8(output).expect("utf8");

        assert_eq!(
            csv_str,
            "client,available,held,total,locked\n\
             1,-1000000.5000,1000000.5000,0.0000,false\n\
             2,0.0000,0.0000,0.0000,false\n"
        );
    }
}
