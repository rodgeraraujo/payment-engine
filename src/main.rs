use payments_engine::engine::Engine;
use payments_engine::processor::{process, write_accounts};
use std::fs::File;
use std::io::{self, BufWriter};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let (Some(path), None) = (args.next(), args.next()) else {
        eprintln!("usage: payments-engine <transactions.csv>");
        return ExitCode::from(2);
    };
    match run(&path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(path: &std::ffi::OsStr) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path).map_err(|e| format!("cannot open {path:?}: {e}"))?;
    let mut engine = Engine::new();
    process(&mut engine, file)?;
    write_accounts(&engine, BufWriter::new(io::stdout().lock()))?;
    Ok(())
}
