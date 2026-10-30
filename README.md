# payments-engine

A small streaming payments engine. It reads transactions from a CSV file,
applies deposits, withdrawals, disputes, resolves and chargebacks, and prints
the final client balances as CSV on stdout.

```
cargo run --release -- transactions.csv > accounts.csv
```

Docker (no local Rust needed):

```
docker build -t payments-engine .
docker run --rm -v "$PWD:/data:ro" payments-engine /data/transactions.csv > accounts.csv
```

Diagnostics (skipped rows, rejected transactions) go to **stderr** only, so
stdout is always clean CSV. Exit codes: `0` ok, `1` I/O failure, `2` bad usage.

## Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): modules, data flow, state machine, scaling.
- [docs/AI_WORKFLOW.md](docs/AI_WORKFLOW.md): how AI was used and how its output was validated, plus a requirement-to-test matrix.
- [docs/PROMPTS.md](docs/PROMPTS.md): prompt log mapped to commits.
- [docs/IMPROVEMENTS.md](docs/IMPROVEMENTS.md): detailed improvements post-baseline, reasons, trade-offs, and verification.

## Project layout

```text
payments-engine/
├── Cargo.toml
├── Makefile
├── Dockerfile
├── README.md
├── docs/                 # Architecture, workflow, task breakdown, improvements
├── scripts/              # Dataset generator for load and scale testing
├── src/
│   ├── lib.rs            # Re-exports, forbid(unsafe_code), warn(missing_docs)
│   ├── main.rs           # CLI argument parsing, buffered I/O, exit codes
│   ├── error.rs          # Strongly-typed error enums
│   ├── domain/           # Pure business logic (zero I/O dependencies)
│   │   ├── amount.rs     # Fixed-point decimal with checked arithmetic
│   │   ├── account.rs    # Client account state and invariant checks
│   │   └── engine.rs     # Transaction processor and dispute state machine
│   └── io/               # Input/Output adapters
│       ├── model.rs      # Raw CSV deserialization
│       └── processor.rs  # Streaming reader, stderr diagnostics, CSV writer
└── tests/
    ├── cli.rs            # End-to-end binary execution against test fixtures
    ├── properties.rs     # Proptest invariants (total == available + held)
    ├── reference_model.rs# Proptest ledger replay comparison against Engine
    ├── robustness.rs     # Fuzzing and hostile parser tests
    └── fixtures/         # Spec and edge-case CSV inputs with expected outputs
```

## Assumptions (where the spec is silent)

| Topic | Decision | Reason |
|---|---|---|
| Disputable transactions | Only **deposits** | Disputing a withdrawal would *increase* funds; a withdrawal id is treated as "not found". |
| Cross-client references | A dispute/resolve/chargeback must come from the **same client** as the original tx, otherwise ignored | Prevents one client touching another's funds. |
| Locked account | Rejects new deposits and withdrawals. Disputes, resolves and chargebacks on already-held funds still process | Bank-like freeze; lets open disputes finish. |
| Dispute after funds were spent | Allowed; `available` can go negative, `total` stays correct | The held amount is owed regardless of what was spent. |
| Re-dispute | A resolved deposit can be disputed again; a charged-back one cannot | Chargeback is final. |
| Duplicate deposit tx id | Ignored | Ids are globally unique per the spec. Duplicate *withdrawal* ids are not tracked (see Efficiency). |
| Amounts | Must be > 0 and have at most 4 decimals; otherwise the row is skipped | Never silently round money. |
| Unknown client | Created on first deposit/withdrawal (a failed withdrawal still yields a zero row) | Spec: "If a client doesn't exist create a new record". |
| Unknown tx type / malformed row | Logged to stderr and skipped, processing continues | One bad row must not lose the whole file. |
| Output | 4 decimal places, sorted by client id | Deterministic and diff-friendly. |

## Correctness

- Money is `rust_decimal::Decimal` wrapped in an `Amount` newtype: exact base-10 maths, no floats, only *checked* operations (overflow is an error, not a panic or wrap).
- `Account` fields are private. `total` is derived (`available + held`), never stored, so the invariant cannot drift.
- Dispute lifecycle is an enum (`Normal`, `Disputed`, `ChargedBack`); illegal transitions return typed errors.
- Input is parsed once into a validated `Transaction` enum; the engine never sees raw strings.

Tests (`cargo test`): unit tests for amount parsing and every engine rule, end-to-end fixtures run through the real binary (`tests/fixtures`), and property tests (`proptest`) checking `total == available + held` and `held >= 0` for random operation sequences.

## Safety and robustness

- No `unsafe`, no `unwrap` on input-derived data, no panics on bad rows.
- Only I/O errors abort the run; everything else is a logged skip.
- Whitespace around fields and missing trailing amount on dispute rows are accepted.

## Efficiency

- Input is streamed row by row (`csv::Reader`) through a 128 KB buffered reader, reducing I/O syscall overhead (~13% time reduction on large inputs).
- Memory grows only with the number of **distinct deposits** (needed for later disputes): one small map entry each. Measured on a generated 5M-row file (134 MB): ~4.9 s and ~160 MB peak RSS (`make bigtest`).
- Withdrawal ids are not stored. The trade-off is that duplicate withdrawal ids cannot be detected; storing them would cost up to ~4 billion set entries in the worst case.
- Scaling to many concurrent streams: `processor::process` is generic over `Read`, so it works on a socket. See ARCHITECTURE.md for the sharding approach.

## Improvements

Detailed improvements made after the initial implementation are documented in [docs/IMPROVEMENTS.md](docs/IMPROVEMENTS.md):
- **Correctness**: Explicit overflow detection on `Account::total()` (`I1`), full reference-model property test (`I6`), direct in-memory stream testing with format edge cases (`I5`).
- **Safety & Robustness**: `#![forbid(unsafe_code)]` and `#![warn(missing_docs)]` compiler enforcement (`I4`), parser fuzz testing against arbitrary byte streams (`I7`), centralized locked account policy (`I9`), and CI dependency vulnerability scanning (`I10`).
- **Efficiency & Observability**: Skip counters aggregated by error category with execution summary on stderr (`I2`, `I3`), 128KB buffered input reducing I/O syscalls (`I8`).
- **Architecture**: Clear layer separation (`src/domain/` with zero I/O imports, `src/io/` for CSV adapters, dedicated `tests/` suites) (`I12`).

## Limitations

1. **Duplicate withdrawal IDs**: To keep memory bounded only by active deposits, withdrawal transaction IDs are not indexed. If an external client retries or duplicates a withdrawal transaction ID with different amounts, the engine processes it based on available balance rather than rejecting it as a duplicate ID.
2. **Unbounded deposit growth**: As long as deposits may be disputed in the future, every deposit transaction must remain indexed in memory (`HashMap<u32, StoredDeposit>`). In a production system processing billions of transactions, this deposit index would be offloaded to an embedded persistent key-value store (e.g. RocksDB) or partitioned across distributed ledger shards.
3. **Single stream execution**: The current binary processes a single sequential CSV file on a single thread. While execution handles ~1.6M rows/second, concurrent multi-tenant ingestion would require sharding by client ID across worker tasks as detailed in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## AI usage

AI assistance was used and is disclosed in [docs/AI_WORKFLOW.md](docs/AI_WORKFLOW.md); the prompt log is in [docs/PROMPTS.md](docs/PROMPTS.md).
