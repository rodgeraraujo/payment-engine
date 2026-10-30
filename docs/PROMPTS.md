# Prompt log

Prompts and design instructions provided to the AI assistant for this project, mapped to the resulting architectural decisions and deliverables. The company name and recruiter contact details are redacted as `[COMPANY]`. The full, unedited chat transcript is stored under `docs/transcript/` with the same redaction.

## Index

| ID | Focus & Objective | Deliverables Produced |
| --- | --- | --- |
| P0 | Specification analysis, ambiguity checks, and boundary validation. | Ambiguity analysis report, edge-case validation checklist, rules audit |
| P1 | Initial scoping, architecture planning, and delivery strategy against take-home specification. | 3-day milestone plan, architectural boundary definition, risk mitigations, test strategy |
| P2 | Implementation of core payments engine, money representation, and streaming I/O. | `Amount` newtype (`rust_decimal`), `Account` invariants, `Engine` dispute state machine, streaming CSV CLI, unit & CLI tests |
| P3 | Documentation suite, system design, and AI disclosure workflow. | `ARCHITECTURE.md`, `IMPLEMENTATION_PLAN.md`, `AI_WORKFLOW.md`, prompt registry |
| P4 | Quality hardening, formal testing, performance optimization, and modular reorganization. | Explicit overflow handling (I1), `#![forbid(unsafe_code)]`, reference-model property test, parser fuzzing, 128KB buffered I/O, `domain`/`io` separation |

---

## Prompt Text

### P0: Specification Analysis, Ambiguity Checks & Verification Criteria

> Perform a thorough technical audit of the attached specification PDF and recruiter guidelines. Before proposing any code or architectural designs, validate the following checks and edge cases:
>
> 1. Ambiguity Analysis:
>    - What happens if a client deposits or withdraws after an account has been locked by a chargeback?
>    - What happens if a dispute occurs on funds that were already withdrawn (can `available` become negative)?
>    - Can resolved transactions be disputed again, or are resolutions final?
>    - Can withdrawals be disputed, or does the dispute mechanism apply exclusively to deposits?
> 2. Boundary & Input Tolerance Checks:
>    - Are whitespaces in headers or around field values permissible?
>    - Does the CSV input format guarantee a trailing comma for dispute/resolve/chargeback rows where the amount is omitted?
>    - How should invalid rows, unknown types, or malformed numerical precision (> 4 decimal places) be handled without crashing the streaming process?
> 3. Output & Grader Purity:
>    - Does the automated grading harness expect `stdout` to contain strictly valid CSV data? Where should warnings and diagnostic logs be routed?
> 4. Scale & State Constraints:
>    - Is it possible to store all historical transaction IDs in memory if the file contains billions of rows, or must in-memory state be strictly bounded by disputable deposits and active client accounts?
>
> Deliver a concise ambiguity report and a list of defensible baseline assumptions that will govern the implementation.

**AI Deliverables**: Edge-case analysis, spec trap identification (negative available balance, locked account semantics, whitespace in CSV, stdout purity), and baseline assumption matrix.

---

### P1: Project Scoping, Architecture, and Delivery Strategy

> We are building a high-throughput, streaming toy payments engine in Rust based on the audited technical specification.
>
> Requirements and constraints:
> 1. **Streaming Input/Output**: Process transactions sequentially from a CSV file via CLI (`cargo run -- <file.csv>`) and write client account balances to `stdout` in CSV format.
> 2. **Memory Efficiency**: Memory consumption must scale strictly with the number of unique client accounts and disputable transactions, never with the total number of rows.
> 3. **Precision & Correctness**: Monetary calculations must maintain exact precision up to 4 decimal places without floating-point inaccuracies.
> 4. **Delivery Target**: The project must include robust test coverage, multi-stage Docker packaging, GitHub Actions CI, and documentation explaining all domain assumptions within a 3-day schedule.
>
> Outline a modular architecture, error strategy, and staged milestone plan for review.

**AI Deliverables**: Structured implementation plan, core domain model proposal, risk mitigation table, and milestone schedule.

---

### P2: Core Implementation & Invariant-Preserving Engine

> Proceed with implementing the core payments engine according to the agreed plan:
>
> 1. **Monetary Representation**: Encapsulate monetary values in an `Amount` newtype wrapping `rust_decimal::Decimal`. Only expose checked operations; reject negative amounts and precision beyond 4 decimal places on input.
> 2. **Account Model**: Keep `Account` fields (`available`, `held`, `locked`) private. Guarantee that `total == available + held` is maintained across all operations (deposit, withdrawal, dispute, resolve, chargeback).
> 3. **Dispute State Machine**: Implement a state machine for deposit lifecycles (`Normal` ⇄ `Disputed` → `ChargedBack`). Reject disputes on withdrawals, cross-client disputes, and re-disputes on charged-back transactions.
> 4. **Streaming Processor**: Stream CSV records using `csv::Reader` with whitespace trimming and flexible record lengths. Skip and log malformed rows to `stderr` without failing the batch; abort only on fatal I/O errors.
> 5. **Quality Gates**: Include unit tests for every business rule, end-to-end CLI integration tests against fixtures, and property-based tests verifying balance invariants.

**AI Deliverables**: Initial functional implementation (`amount`, `account`, `engine`, `processor`, `main`), comprehensive unit tests, CLI fixtures, and invariant property tests.

---

### P3: System Architecture & AI Governance Documentation

> Produce comprehensive project documentation detailing the system design, implementation decisions, and AI governance:
>
> 1. **Architecture (`ARCHITECTURE.md`)**: Document component boundaries, data flow pipelines, the deposit state machine, and a concrete proposal for scaling to concurrent sharded streams.
> 2. **Implementation Plan (`IMPLEMENTATION_PLAN.md`)**: Break down each phase from foundational setup to final review, detailing deliverables, acceptance criteria, and explicit test mappings.
> 3. **AI Workflow & Governance (`AI_WORKFLOW.md`)**: Disclose generative AI usage, outline the verification workflow (human review of diffs, linting, property tests), and provide a requirement-to-test traceability matrix.
> 4. **Prompt Log**: Initialize a prompt registry mapping user requests to design choices and committed code.

**AI Deliverables**: `ARCHITECTURE.md`, `IMPLEMENTATION_PLAN.md`, `AI_WORKFLOW.md`, and initial `PROMPTS.md`.

---

### P4: Quality Hardening, Modular Reorganization, and Performance Optimization

> Execute a comprehensive improvement and refactoring round on the working codebase following `docs/IMPROVEMENTS.md`:
>
> 1. **Correctness & Safety**:
>    - Remove silent zero fallbacks in `Account::total()`, returning explicit `Option<Amount>` to detect arithmetic overflow.
>    - Add `#![forbid(unsafe_code)]` and `#![warn(missing_docs)]` with rustdoc examples across public interfaces.
>    - Centralize the locked-account policy in `ensure_unlocked`.
> 2. **Verification & Testing**:
>    - Implement an independent reference model that replays transactions chronologically, and verify state equivalence via property testing (`proptest`).
>    - Add parser robustness tests feeding arbitrary byte streams and malformed CSV rows to prove no input can trigger a panic.
>    - Test in-memory streaming readers (`&[u8]`) and output formatting edge cases directly.
> 3. **Efficiency & Observability**:
>    - Replace per-row stderr logging with category-aggregated diagnostics (`Stats`) and an execution summary on `stderr`.
>    - Buffer input streams using a 128KB `BufReader` and measure the performance gain on large datasets.
>    - Add automated dependency vulnerability scanning (`cargo-audit`) to CI.
> 4. **Modular Architecture**:
>    - Decouple domain logic from I/O by splitting the crate into `src/domain/` (zero I/O dependencies) and `src/io/` (deserialization and streaming processor).

**AI Deliverables**: Refactored modular layout (`src/domain/`, `src/io/`), expanded test suite (`tests/reference_model.rs`, `tests/robustness.rs`), aggregated stderr diagnostics, optimized buffered I/O, and synchronized documentation.

---

## Decisions that Came from Prompts

- **Specification Verification**: Identified specification traps upfront (cross-client dispute fraud, negative available balance on spent funds, memory scale constraints) and codified them into explicit assumptions (P0).
- **Decimal Representation**: Selected `rust_decimal` behind an `Amount` newtype to guarantee exact base-10 arithmetic and compile-time prevention of floating-point inaccuracies (P1, P2).
- **Deposit-Only Disputes**: Restricted dispute lifecycle strictly to deposits, treating withdrawal disputes as invalid to prevent unauthorized balance inflation (P0, P1, P2).
- **Bank-Style Account Freeze**: Chargebacks lock the account, blocking new deposits and withdrawals while permitting active dispute resolutions to finalize (P0, P2, P4).
- **Pure `stdout` Stream**: Strict separation between data output on `stdout` and diagnostic/metrics logging on `stderr` (P0, P1, P2, P4).
- **Architectural Layering**: Enforced that `domain` contains zero dependencies on `io`, `serde`, or `csv`, ensuring domain logic can be reused in non-file contexts (P4).

---

## AI Suggestion Review

| Suggestion | Source | Decision | Technical Justification |
| --- | --- | --- | --- |
| Upfront ambiguity analysis | P0 | Accepted | Uncovers spec contradictions and traps before code generation. |
| `rust_decimal` behind an `Amount` newtype | P2 | Accepted | Guarantees exact base-10 financial math and checked arithmetic. |
| Restrict disputes to deposits | P0, P2 | Accepted | Disputing a withdrawal would erroneously add funds back to an account. |
| Bank-freeze locked account policy | P0, P2 | Accepted | Blocks further balance movement while allowing in-flight disputes to settle. |
| Property-based testing with `proptest` | P2 | Accepted | Validates core mathematical invariants across randomized operation sequences. |
| Do not index duplicate withdrawal IDs | P0, P2 | Accepted | Avoids unbounded memory growth (up to ~4B records) while meeting spec requirements. |
| Explicit `Option<Amount>` on `Account::total()` | P4 | Accepted | Eliminates silent zero fallback on potential arithmetic overflow. |
| Centralize lock enforcement in `ensure_unlocked` | P4 | Accepted | Isolates business policy to a single point of change. |
| `#![forbid(unsafe_code)]` and doctests | P4 | Accepted | Enforces memory safety at the compiler level and provides verified API documentation. |
| Naive replay ledger reference model | P4 | Accepted | Proves engine state machine correctness against an independent reference implementation. |
| Arbitrary byte and CSV fuzz testing | P4 | Accepted | Empirically verifies parser robustness and absence of panics on corrupt input. |
| Category-aggregated stderr metrics | P4 | Accepted | Eliminates per-row I/O bottlenecks during large batch ingestion. |
| 128KB `BufReader` on input streams | P4 | Accepted | Reduces file system syscall overhead, yielding ~13% measured throughput improvement. |
| Modular `domain/` and `io/` separation | P4 | Accepted | Enforces clean dependency inversion; core business logic has zero I/O imports. |

---

## Template for New Entries

```markdown
| Pn | <objective and constraints> | <deliverables produced> |
```
