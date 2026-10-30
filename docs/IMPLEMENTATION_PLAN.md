# Implementation plan

A toy streaming payments engine in Rust. Reads transactions from a CSV file,
applies deposits, withdrawals, disputes, resolves and chargebacks, and prints
client balances as CSV on stdout.

```
cargo run -- transactions.csv > accounts.csv
```

## 1. Goals and constraints

- The CLI above is mandatory; anything else is ignored by reviewers.
- A plain `cargo new` crate that builds with `cargo build` and runs with `cargo run`.
- Scoring areas: basics, completeness, correctness, safety and robustness, efficiency, maintainability. Clean code beats clever code.
- Input: columns `type, client, tx, amount`; `client` is `u16`, `tx` is `u32`, amounts have up to 4 decimals. Whitespace must be accepted.
- Output: `client, available, held, total, locked`, 4-decimal precision, row order irrelevant.
- Rows are chronological. Client ids and tx ids are not ordered. Tx ids are globally unique.
- The submission must not mention the company or its products, and the test file must not be committed.
- AI use must be disclosed, with the full prompt transcript included.
- If the spec is inconsistent, the input/output files take precedence. Document every assumption.

## 2. Key decisions

| Topic | Decision | Reason |
| --- | --- | --- |
| Money type | `rust_decimal` inside an `Amount` newtype | Exact base-10 maths, checked operations, little custom code |
| I/O | Stream with `csv` + `serde`, one row at a time | Constant memory per row |
| State | `HashMap<u16, Account>` and `HashMap<u32, StoredDeposit>` | Only deposits can be disputed |
| Errors | Typed enums with `thiserror`; bad rows are logged and skipped | One bad row must not lose the file |
| Disputes | Deposits only | Disputing a withdrawal would add funds |
| Ownership | A dispute must come from the same client as the original tx | Prevents touching another client's funds |
| Locked account | Rejects deposits and withdrawals; disputes, resolves and chargebacks still process | Bank-like freeze |
| Output | Sorted by client id, 4 decimals | Deterministic |
| Logging | stderr only | stdout must stay pure CSV |

## 3. Architecture

```
main.rs        CLI, args, exit codes
processor.rs   CSV stream in -> engine -> CSV out, logs skipped rows
model.rs       RawRecord -> validated Transaction
engine.rs      business rules, no I/O
account.rs     balances and locked flag, invariant-preserving methods
amount.rs      Decimal newtype with checked arithmetic
error.rs       typed errors shared by all layers
```

Dependencies only point downwards. The engine has no I/O, so it can be tested
in memory and reused in a server.

Dispute state machine (per deposit):

```
Normal --dispute--> Disputed --resolve----> Normal
                       |
                       +------chargeback--> ChargedBack (account locked, final)
```

| Operation | available | held | total |
| --- | --- | --- | --- |
| deposit | +a | | +a |
| withdrawal (if available >= a) | -a | | -a |
| dispute | -a | +a | = |
| resolve | +a | -a | = |
| chargeback | | -a | -a, lock |

## 4. Phases

Each phase has a deliverable and an acceptance check. Do them in order.

### Phase 0: foundation

- Create the crate and add dependencies: `csv`, `serde`, `rust_decimal`, `thiserror`; dev: `assert_cmd`, `proptest`.
- Add `.gitignore` (including `*.pdf`).
- Accept: `cargo build` succeeds.

### Phase 1: errors

- `AmountError`, `ParseError`, `EngineError` with clear messages.
- Accept: compiles; no module depends on a layer above it.

### Phase 2: amount

- `Amount::parse_positive`: strictly positive, at most 4 decimals, trims whitespace.
- Checked add and subtract; `Display` always prints 4 decimals.
- Accept: unit tests for valid, too precise, zero, negative, non-numeric, empty, overflow.

### Phase 3: account

- Private fields; `total` is derived, never stored.
- Methods: deposit, withdraw, hold, release, chargeback; all checked.
- Accept: `total == available + held` always.

### Phase 4: model

- `RawRecord` (amount optional) and `Transaction` enum.
- `TryFrom<RawRecord>`: validates type and amount presence; ignores an amount on dispute-like rows.
- Accept: unknown type and missing amount produce typed errors.

### Phase 5: engine

- `apply(Transaction)` for all five types, with the rules in section 2.
- Accept: unit tests for each rule (see section 5).

### Phase 6: processor and CLI

- Reader: trim all fields, flexible record length (dispute rows have no amount).
- Skip and log malformed rows; abort only on I/O errors.
- Writer: header plus sorted rows, 4 decimals.
- `main.rs`: exactly one file argument; exit 0 ok, 1 I/O failure, 2 bad usage.
- Accept: the spec example produces the expected output.

### Phase 7: tests

- End-to-end fixtures run through the real binary.
- Property tests for invariants.
- Accept: `cargo test` passes; stdout is pure CSV.

### Phase 8: tooling and delivery

- Large dataset generator and a scale check.
- Multi-stage `Dockerfile`, CI workflow (fmt, clippy, tests, docker build), `Makefile`.
- README with assumptions; architecture doc; AI workflow doc; prompt log.
- Accept: fresh clone builds and runs; `cargo fmt --check` and `cargo clippy -D warnings` clean.

## 5. Test plan

Unit tests:

- Amount parsing and overflow.
- Deposit and withdrawal; insufficient funds leaves balance unchanged.
- Dispute then resolve; dispute then chargeback and lock.
- Locked account rejects deposits and withdrawals.
- Dispute after funds were spent: available negative, total consistent.
- Unknown tx, wrong client, resolve or chargeback without a dispute, double dispute.
- Withdrawals cannot be disputed; duplicate deposit id ignored.
- Charged-back deposit cannot be disputed again; resolved deposit can.

End-to-end fixtures (expected outputs computed by hand, not copied from the engine):

- `basic`: the example from the spec.
- `disputes`: dispute, resolve, chargeback, post-lock deposit, open dispute.
- `malformed`: bad decimals, negatives, unknown type, bad ids, odd whitespace.
- CLI: missing argument gives exit 2; missing file gives exit 1; diagnostics only on stderr.

Property tests: for any sequence of operations, `total == available + held` and `held >= 0`, with no panics.

Scale check: generate 5M rows, record time and peak memory.

## 6. AI validation workflow

1. Plan and agree the design and spec ambiguities before any code.
2. Work in small steps; each step builds and passes tests.
3. Treat the spec as the source of truth, not the AI output.
4. AI generates, a human reads every diff and runs fmt, clippy and tests.
5. Ask the AI to find spec mismatches and hostile inputs; turn each finding into a test or a documented assumption.
6. Hand-compute expected fixture outputs.
7. Run the scale check.

Per-step checklist:

- Matches the spec wording, not just the example?
- Any `unwrap`, `expect` or indexing that input can trigger?
- Any float or unchecked arithmetic on money?
- Is every new rule covered by a test that fails without it?
- Does stdout stay pure CSV?
- No reference to the company or the test file anywhere?

## 7. Requirement-to-test matrix

| Requirement | Where | Test |
| --- | --- | --- |
| CLI `cargo run -- file > out` | `main.rs` | basic fixture |
| Whitespace and 4-decimal precision | `processor.rs`, `amount.rs` | malformed fixture, amount tests |
| Withdrawal fails without funds | `account.rs` | insufficient funds test |
| Dispute holds funds, total unchanged | `engine.rs` | dispute then resolve |
| Unknown tx ignored | `engine.rs` | invalid references test |
| Resolve needs an active dispute | `engine.rs` | invalid references test |
| Chargeback reduces held and total, locks | `engine.rs` | chargeback test |
| total = available + held | `account.rs` | property tests |
| Streaming, large input | `processor.rs` | scale check |
| Errors handled, no panics | everywhere | malformed fixture, property tests |

## 8. Assumptions to document in the README

- Only deposits are disputable.
- Cross-client references are ignored.
- Locked accounts reject new deposits and withdrawals; open disputes can still finish.
- A dispute after spending can make `available` negative.
- A resolved deposit can be re-disputed; a charged-back one cannot.
- Duplicate deposit ids are ignored; duplicate withdrawal ids are not tracked.
- Amounts must be positive with at most 4 decimals, otherwise the row is skipped.
- A client is created on its first deposit or withdrawal.
- Malformed rows are logged to stderr and skipped.

## 9. Risks

| Risk | Mitigation |
| --- | --- |
| Locked-account behaviour differs from the grader | Document the choice; keep it in one place so it is easy to flip |
| Large input exhausts memory | Store only deposits, with a small struct per entry |
| Hidden formatting differences in output | Always 4 decimals, header always printed, sorted rows |
| Panics on odd input | No `unwrap` on input-derived data; typed errors; property tests |
| Company name leaks into the repo | Search the repo and history for it before submitting |

## 10. Three-day schedule

- Day 1: phases 0 to 5 (foundation, errors, amount, account, model, engine) with unit tests.
- Day 2: phases 6 and 7 (processor, CLI, fixtures, property tests, scale check).
- Day 3: phase 8 (Docker, CI, README, architecture, AI documents) and a final review.

## 11. Submission checklist

- [ ] `cargo build`, `cargo test`, `cargo fmt --check`, `cargo clippy -D warnings` all pass.
- [ ] The spec example gives the expected output.
- [ ] README lists assumptions, correctness approach, safety, efficiency.
- [ ] AI usage disclosed; transcript included with the company name redacted.
- [ ] No mention of the company, its brands or domains in code, docs or history.
- [ ] The test PDF is not in the repo.
- [ ] Repo is public.
- [ ] Link sent to the recruiter with the admin address in cc, before the deadline.

## 12. Improvement round

After the first working version, a second round raises quality without
changing CLI behavior. The full list, reasons and trade-offs are in
`IMPROVEMENTS.md`; the small tasks are in `TASKS.md` (phases 12 to 14).

| Area | Planned change | Why |
| --- | --- | --- |
| Correctness | Explicit overflow in `total()`, reference-model property test, direct `process` tests | Remove a silent failure; prove results, not only invariants |
| Safety | `forbid(unsafe_code)`, robustness test, dependency audit in CI | Turn claims into checks |
| Efficiency | Aggregated skip logging; measured buffer, hasher and compact deposit | Cut logging cost; adopt changes only with numbers |
| Maintainability | `domain/` and `io/` modules, docs and scripts folders, rustdoc examples | Make layers and dependency direction visible |
| Basics | README limitations, layout section, docs kept in sync | A reviewer can trust the documentation |

Order: correctness and safety first, then efficiency, then reorganization,
then documentation. Moves and renames are kept separate from behavior changes.
