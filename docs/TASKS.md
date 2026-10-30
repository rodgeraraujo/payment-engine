# Task breakdown

The smallest useful tasks, derived from the spec, the recruiter email and the
implementation plan. Each task is meant to take 5 to 30 minutes and has a clear
"done when". Do them in order inside a phase; phases build on each other.

Legend: `[ ]` to do. Size: S (under 10 min), M (10 to 30 min).

## Phase 0: setup

- [x] T0.1 (S) Create the crate with `cargo new payments-engine`. Done when `cargo run` prints the default message.
- [x] T0.2 (S) Add dependencies: `csv`, `serde` (derive), `rust_decimal`, `thiserror`. Done when `cargo build` passes.
- [x] T0.3 (S) Add dev-dependencies: `assert_cmd`, `proptest`. Done when `cargo test` still runs.
- [x] T0.4 (S) Add `.gitignore` with `/target` and `*.pdf`. Done when the spec PDF is ignored by git.
- [x] T0.5 (S) Split into `lib.rs` (modules) and `main.rs` (CLI only). Done when both compile.

## Phase 1: errors

- [x] T1.1 (S) Define `AmountError`: invalid number, not positive, too many decimals. Done when each variant has a readable message.
- [x] T1.2 (S) Define `ParseError`: unknown type, missing amount, wrapped `AmountError`.
- [x] T1.3 (S) Define `EngineError`: insufficient funds, account locked, duplicate tx, unknown tx, client mismatch, invalid state, overflow.
- [x] T1.4 (S) Check no `unwrap` or `panic!` exists in the error module.

## Phase 2: amount

- [x] T2.1 (S) Create the `Amount` newtype around `Decimal`. Done when it derives `Copy`, `Eq`, `Ord`, `Default`.
- [x] T2.2 (M) Implement `parse_positive`: trim spaces, reject non-numbers. Done when `" 2 "` parses and `"abc"` fails.
- [x] T2.3 (S) Reject zero and negative values.
- [x] T2.4 (S) Reject more than 4 decimals, but accept `1.50000` (trailing zeros are fine).
- [x] T2.5 (S) Implement `checked_add` and `checked_sub`. Done when overflow returns `None`.
- [x] T2.6 (S) Implement `Display` with exactly 4 decimals (`1.5` prints `1.5000`).
- [x] T2.7 (M) Unit tests: valid values, each rejection, overflow.

## Phase 3: account

- [x] T3.1 (S) Create `Account` with private `available`, `held`, `locked`. Done when fields are not public.
- [x] T3.2 (S) Add getters; implement `total()` as `available + held` (never stored).
- [x] T3.3 (S) `deposit`: reject if locked, use checked maths.
- [x] T3.4 (S) `withdraw`: reject if locked or `available < amount`; balance unchanged on failure.
- [x] T3.5 (S) `hold`: move available to held, total unchanged.
- [x] T3.6 (S) `release`: move held back to available, total unchanged.
- [x] T3.7 (S) `chargeback`: remove from held and set `locked = true`.

## Phase 4: model

- [x] T4.1 (S) Define `RawRecord` with `type`, `client: u16`, `tx: u32`, `amount: Option<String>`.
- [x] T4.2 (S) Define the `Transaction` enum with the five variants.
- [x] T4.3 (M) Implement `TryFrom<RawRecord>`: lowercase and trim the type, require an amount for deposit and withdrawal.
- [x] T4.4 (S) Ignore any amount on dispute, resolve and chargeback rows.
- [x] T4.5 (S) Treat an empty amount string as missing.
- [x] T4.6 (S) Unit tests: each type, unknown type, missing amount.

## Phase 5: engine

- [x] T5.1 (S) Create `Engine` with the accounts map and the deposits map.
- [x] T5.2 (S) Define `DisputeState`: `Normal`, `Disputed`, `ChargedBack`.
- [x] T5.3 (S) Define `StoredDeposit { client, amount, state }`.
- [x] T5.4 (M) `deposit`: reject duplicate tx id; create the account if missing; store the deposit.
- [x] T5.5 (S) `withdraw`: create the account if missing; delegate to `Account`.
- [x] T5.6 (M) Helper to look up a deposit: unknown tx and client mismatch errors.
- [x] T5.7 (S) Helper to check the expected dispute state.
- [x] T5.8 (S) `dispute`: only from `Normal`; hold funds; set `Disputed`.
- [x] T5.9 (S) `resolve`: only from `Disputed`; release funds; set `Normal`.
- [x] T5.10 (S) `chargeback`: only from `Disputed`; reduce held, lock, set `ChargedBack`.
- [x] T5.11 (S) `apply(Transaction)` that dispatches to the five handlers.
- [x] T5.12 (S) `accounts_sorted()` returning rows ordered by client id.
- [x] T5.13 (M) Unit tests, one per rule:
  - [x] deposit and withdrawal
  - [x] insufficient funds leaves balance unchanged
  - [x] dispute then resolve
  - [x] chargeback freezes and blocks deposits and withdrawals
  - [x] dispute after spending makes available negative, total correct
  - [x] unknown tx, wrong client, resolve or chargeback without a dispute, double dispute
  - [x] withdrawals cannot be disputed
  - [x] duplicate deposit id ignored
  - [x] charged-back deposit cannot be re-disputed; resolved one can

## Phase 6: processor and CLI

- [x] T6.1 (S) Build the CSV reader with trim for all fields. Done when `type, client, tx, amount` headers work.
- [x] T6.2 (S) Enable flexible record length. Done when `dispute,1,1` (no trailing comma) parses.
- [x] T6.3 (M) Loop over `deserialize::<RawRecord>()` one row at a time (no collecting into a vector).
- [x] T6.4 (S) On a malformed row: log to stderr with the line number and continue.
- [x] T6.5 (S) On an I/O error: stop and return the error.
- [x] T6.6 (S) Convert each row to `Transaction`; on error log and continue.
- [x] T6.7 (S) Apply each transaction; on `EngineError` log and continue.
- [x] T6.8 (S) Return simple stats (applied, skipped) for tests.
- [x] T6.9 (M) `write_accounts`: header `client,available,held,total,locked`, sorted rows, 4 decimals.
- [x] T6.10 (S) `main`: exactly one argument, otherwise print usage and exit 2.
- [x] T6.11 (S) `main`: open the file; on failure print the error and exit 1.
- [x] T6.12 (S) `main`: write the output through a buffered stdout. Done when `cargo run -- file > out.csv` works.

## Phase 7: tests

- [x] T7.1 (S) Fixture `basic.csv` from the spec example; hand-computed expected output.
- [x] T7.2 (M) Fixture `disputes.csv`: dispute, resolve, chargeback, a deposit after lock, an open dispute; hand-computed output.
- [x] T7.3 (M) Fixture `malformed.csv`: too many decimals, negative, text amount, missing amount, unknown type, bad ids, out-of-range client, odd whitespace.
- [x] T7.4 (S) CLI test: basic fixture matches the expected output exactly.
- [x] T7.5 (S) CLI test: disputes fixture.
- [x] T7.6 (S) CLI test: malformed fixture exits successfully and keeps valid rows.
- [x] T7.7 (S) CLI test: diagnostics only on stderr; stdout starts with the header.
- [x] T7.8 (S) CLI test: no argument exits with code 2.
- [x] T7.9 (S) CLI test: missing file exits with code 1.
- [x] T7.10 (M) Property test generator for random transactions (few clients, few tx ids, amounts with 4 decimals).
- [x] T7.11 (S) Property: `total == available + held` after any sequence.
- [x] T7.12 (S) Property: `held >= 0` after any sequence.

## Phase 8: quality gates

- [x] T8.1 (S) `cargo fmt --check` passes.
- [x] T8.2 (S) `cargo clippy --all-targets -- -D warnings` passes.
- [x] T8.3 (S) Search the code for `unwrap`, `expect`, `panic!`, `unsafe`; justify or remove each.
- [x] T8.4 (M) Write a script that generates a large CSV (millions of rows, mixed types).
- [x] T8.5 (S) Run the large file in release mode; record time and peak memory.
- [x] T8.6 (S) Check that memory grows with distinct deposits only, not with total rows.

## Phase 9: delivery tooling

- [x] T9.1 (M) Multi-stage `Dockerfile` (build image, slim runtime image, entrypoint is the binary).
- [x] T9.2 (S) `.dockerignore` with `target`, `.git`, `*.pdf`.
- [x] T9.3 (S) Run the container against a fixture and compare with the expected output.
- [x] T9.4 (S) `Makefile` targets: build, test, lint, run, docker, big test.
- [x] T9.5 (M) CI workflow: fmt, clippy, tests, docker build.

## Phase 10: documentation

- [x] T10.1 (M) README: what it does, how to run it, Docker usage, exit codes, stdout vs stderr.
- [x] T10.2 (M) README: assumptions table (disputes on deposits only, client ownership, locked account, negative available, re-dispute, duplicates, amount rules, client creation, bad rows).
- [x] T10.3 (S) README: correctness section (Decimal newtype, private fields, enum state machine, parse at the edge).
- [x] T10.4 (S) README: safety and robustness (no unsafe, no panics on input, error policy).
- [x] T10.5 (S) README: efficiency (streaming, memory trade-off, measured numbers).
- [x] T10.6 (S) README: how it would scale to many concurrent streams.
- [x] T10.7 (M) Architecture doc: layers, data flow, dispute state machine, balance table.
- [x] T10.8 (M) AI workflow doc: disclosure, validation steps, per-step checklist, requirement-to-test matrix.
- [x] T10.9 (S) Prompt log with verbatim prompts and a review of each AI suggestion.
- [ ] T10.10 (S) Export the full chat transcript into `docs/transcript/`.
- [ ] T10.11 (S) Redact the company name and contact details in every document and the transcript.

## Phase 11: final review and submission

- [x] T11.1 (S) Fresh clone: `cargo build` and `cargo test` pass from scratch.
- [x] T11.2 (S) Run the spec example and diff against the expected output.
- [x] T11.3 (S) Search the whole repo and history for the company name, brands and domains. Zero results.
- [x] T11.4 (S) Confirm the test PDF is not in the repo or the history.
- [x] T11.5 (S) Re-read the spec's scoring table and tick each category against the README.
- [ ] T11.6 (S) Be ready to explain each assumption and each AI-assisted decision (needed for the interview).
- [ ] T11.7 (S) Make the GitHub repo public.
- [ ] T11.8 (S) Email the repo link to the recruiter with the admin address in cc, before the deadline.
- [ ] T11.9 (S) Confirm receipt as the email asks.

## Phase 12: improvement round

Details and reasons for each item are in `IMPROVEMENTS.md` (IDs I1 to I11).

- [x] T12.1 (M) I1: make `Account::total()` explicit about overflow instead of falling back to zero. Why: a silent zero would print a wrong total. Done when an overflow test fails on the old behavior.
- [x] T12.2 (S) I2: print an applied/skipped summary on stderr. Why: the counters were unused. Done when a CLI test sees the summary.
- [x] T12.3 (M) I3: count skipped rows by error kind and print the totals. Why: per-row logging dominated time on large files. Done when a large run is faster and the summary is correct.
- [x] T12.4 (S) I4: add `forbid(unsafe_code)` and `warn(missing_docs)`. Why: make the safety claim enforced by the compiler.
- [x] T12.5 (M) I4: document every public item and add doc-test examples to `Amount::parse_positive` and `Engine::apply`. Done when `cargo test --doc` passes.
- [x] T12.6 (M) I5: test `process` with `&[u8]` input. Why: proves it works with any reader, not only files.
- [x] T12.7 (S) I5: test output edge cases (negative available, large values, only a failed withdrawal, trailing zeros).
- [x] T12.8 (M) I6: write a naive reference model that replays the full history for each query.
- [x] T12.9 (M) I6: property test comparing the engine with the reference model on random sequences. Why: invariants alone do not prove results are right.
- [x] T12.10 (M) I7: robustness test with random byte strings and CSV-shaped input. Done when nothing panics and stdout stays CSV.
- [x] T12.11 (S) I8: record baseline time and peak memory for the 5M-row file.
- [x] T12.12 (M) I8: try a larger read buffer, a faster hasher and a compact stored deposit, one at a time, measuring each.
- [x] T12.13 (S) I8: keep only the changes that improve the numbers; note the results in the README.
- [x] T12.14 (S) I9: keep the locked-account rule in one function with a comment explaining the assumption.
- [x] T12.15 (S) I10: add a dependency audit job to CI.
- [x] T12.16 (S) I11: add the "Limitations" section to the README.

## Phase 13: reorganization

Reason (I12): make the layers and dependency direction visible. Do moves
separately from behavior changes; run all tests after each move.

- [x] T13.1 (S) Create `src/domain/` and move `amount`, `account`, `engine` into it.
- [x] T13.2 (S) Create `src/io/` and move `model`, `processor` into it.
- [x] T13.3 (S) Keep `error.rs` at the crate root; update `lib.rs` re-exports.
- [x] T13.4 (S) Make internal items `pub(crate)`; keep the public API minimal. Why: `domain` must not depend on `io`.
- [x] T13.5 (S) Move scripts to `scripts/` and docs to `docs/` (including `transcript/`).
- [x] T13.6 (S) Split tests into `cli.rs`, `properties.rs`, `reference_model.rs`, `robustness.rs`, with `fixtures/`.
- [x] T13.7 (S) Update paths in README links, Makefile, Dockerfile and CI.
- [x] T13.8 (S) Run all quality gates after the final move.

## Phase 14: keep documentation in sync (I13)

- [x] T14.1 (S) Update `docs/ARCHITECTURE.md` for the new layout and module names.
- [x] T14.2 (S) Add a "Project layout" section to the README.
- [x] T14.3 (S) Add the improvement summary and the reasons to the README (link to `IMPROVEMENTS.md`).
- [x] T14.4 (S) Update the requirement-to-test matrix in `docs/AI_WORKFLOW.md` with the new tests.
- [x] T14.5 (S) Update `docs/IMPLEMENTATION_PLAN.md` and tick finished items in this file.
- [x] T14.6 (S) Add the improvement prompt to `docs/PROMPTS.md` and review each AI suggestion (accepted, changed, rejected, why).
- [x] T14.7 (S) Re-run the final review checklist from phase 11.

## Suggested split over three days

- Day 1: phases 0 to 5.
- Day 2: phases 6 to 8.
- Day 3: phases 9 to 11, then the improvement round (phases 12 to 14) if time allows.
