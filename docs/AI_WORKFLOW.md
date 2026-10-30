# AI usage and validation workflow

## Disclosure

Generative AI (Claude) was used for planning, code generation, tests and
documentation. The author reviewed every change, can explain each decision, and
owns the final code. The prompt log is in [PROMPTS.md](PROMPTS.md).

## Workflow

1. **Plan first.** Agree the design and the ambiguous-spec decisions before any code.
2. **Small steps.** They all as separated by commit per step; each commit builds and passes tests.
3. **Spec as source of truth.** Every rule is traced to the requirement text (table below); AI output is never trusted over the spec.
4. **AI generates, human verifies.** For each step: read the diff, run `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`.
5. **Adversarial review.** Ask the AI to find spec mismatches and to propose hostile inputs; turn each finding into a test or a documented assumption.
6. **Independent check.** Hand-compute expected outputs for fixtures (do not copy engine output into expected files).
7. **Scale check.** `make bigtest` for time and memory on 5M rows.

## Per-step checklist

- [ ] Does it match the spec wording (not just the example)?
- [ ] Any `unwrap`/`expect`/indexing that input can trigger?
- [ ] Any float or unchecked arithmetic on money?
- [ ] Is every new rule covered by a test that fails without it?
- [ ] Does stdout stay pure CSV?

## Requirement to test matrix

| Requirement | Where | Test |
| --- | --- | --- |
| CLI `cargo run -- file > out` | `main.rs` | `tests/cli.rs::basic_example_from_the_spec` |
| Whitespace and 4-decimal precision accepted | `io/processor.rs`, `domain/amount.rs` | `malformed.csv` fixture, `amount::tests` |
| Withdrawal fails without funds | `domain/account.rs` | `insufficient_funds_leaves_balance_unchanged` |
| Dispute holds funds, total unchanged | `domain/engine.rs` | `dispute_then_resolve` |
| Unknown tx ignored | `domain/engine.rs` | `invalid_references_are_rejected` |
| Resolve needs an active dispute | `domain/engine.rs` | `invalid_references_are_rejected` |
| Chargeback reduces held and total, locks | `domain/engine.rs` | `chargeback_freezes_and_blocks_further_activity` |
| total = available + held (checked, no silent overflow) | `domain/account.rs` | `properties.rs`, `account::tests::total_returns_none_on_overflow` |
| Engine behavior matches reference replay ledger | `domain/engine.rs` | `tests/reference_model.rs` |
| In-memory stream processing & formatting edge cases | `io/processor.rs` | `processor::tests::process_works_with_in_memory_slice`, `output_edge_cases_formatting` |
| Robustness: no panics on arbitrary/hostile bytes | `io/processor.rs` | `tests/robustness.rs` |
| Diagnostic summary on stderr, pure CSV on stdout | `main.rs`, `io/processor.rs` | `tests/cli.rs::stderr_summary_reports_applied_and_skipped_counts` |
| Streaming, large input | `io/processor.rs` | `make bigtest` |
| Errors handled, no panics | everywhere | `malformed_rows_are_skipped_not_fatal`, property & robustness tests |
