//! I/O layer: streams CSV in, applies each row to the [`Engine`], writes CSV out.
//! Generic over `Read`/`Write` so it works for files, stdin, or (later) sockets,
//! and is trivially testable with in-memory buffers.

use crate::engine::Engine;
use crate::model::{RawRecord, Transaction};
use std::io::{Read, Write};

/// Counters returned for observability / tests.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub applied: u64,
    pub skipped: u64,
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
                eprintln!("line {line}: skipped malformed row: {e}");
                stats.skipped += 1;
                continue;
            }
        };
        let outcome = Transaction::try_from(raw)
            .map_err(|e| e.to_string())
            .and_then(|t| engine.apply(t).map_err(|e| e.to_string()));
        match outcome {
            Ok(()) => stats.applied += 1,
            Err(reason) => {
                eprintln!("line {line}: skipped: {reason}");
                stats.skipped += 1;
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
        writer.write_record([
            id.to_string(),
            account.available().to_string(),
            account.held().to_string(),
            account.total().to_string(),
            account.is_locked().to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}
