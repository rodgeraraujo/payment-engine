# Prompt log

Prompts given to the AI assistant (Claude) for this project, mapped to what they
produced. The company name and recruiter contact details are redacted as
`[COMPANY]`. Prompt text is verbatim, typos included. The full, unedited
transcript should be exported from the chat and stored under `docs/transcript/`
with the same redaction.

## Index

| ID | Prompt (summary) | Output |
| --- | --- | --- |
| P1 | Spec and recruiter email attached. Asked for documentation, an AI validation workflow, tests and Docker, within 3 days, following all rules. | Plan: rules checklist, design, assumptions, test strategy, 3-day schedule |
| P2 | "Choose the best option"; tell me if there is any trick in the test; give me all the architecture; map the prompts. | Chose `rust_decimal` behind an `Amount` newtype; implementation, tests, Docker, CI; list of spec traps; `ARCHITECTURE.md`; first prompt log |
| P3 | Asked for a graph of the architecture. | Layered module diagram (main, processor, model, engine, account, amount, error) |
| P4 | Asked for a more high-level system design that can be hand-drawn. | 6-box pipeline diagram with drawing steps and talking points |
| P5 | Asked for all project prompts plus the full implementation plan as a separate markdown. | This log and `IMPLEMENTATION_PLAN.md` |
| P6 | Improve project per IMPROVEMENTS.md (correctness, safety, efficiency, maintainability, reorganization). | Quality improvements I1-I13, expanded test suite, modular reorganization |

## Prompt text

### P1

Attachments: the take-home spec (PDF) and the recruiter email with the AI
policy. Neither is reproduced here; the spec file must not be committed.

> I need to implement this, help i need all documentation, workflow for ai validate, test, if posible docker or similar to running it, i have 3 days to implement. Following all the rules present on the documentations.

AI response: plan only, no code, requested review before implementation.

### P2
>
> Choose the best option, if there any trick on the coding test, also give me all the archutecture used in the solution, also the prompts we can map in a documented way too

AI response: picked `rust_decimal`; listed the traps in the spec (whitespace in
headers, optional amount, stdout purity, cross-client disputes, negative
available after a dispute, locked-account ambiguity, scale hint); built the
implementation, tests, scripts, Docker, CI and docs.

### P3
>
> can u give me a graph of the architecture?

### P4
>
> a architecture draw (more high level) so i candraw it on my notepad? like the system design?

### P5
>
> give me all the prompts for the project, attached part of the documentations, so i can add a complete prompt log

Follow-up refinement:

> not need the commiit flow, change my name commit... just the thing related to the project, and also the complete plan implementation, in separated markdown

### P6

> You are helping me finish a Rust take-home: a toy streaming payments engine.
> The repo already works and all tests pass. Improve it and make its structure
> clearer, following the attached coding test specification at every step. Do not
> change what a user or an automated grader can observe (CLI, CSV format, output
> precision, exit codes).
>
> ## How we work
>
> 1. Read the whole repo and the documents in docs/ first (IMPROVEMENTS.md,
>    TASKS.md, IMPLEMENTATION_PLAN.md, ARCHITECTURE.md, AI_WORKFLOW.md). Run the
>    tests, fmt and clippy, and report the baseline.
> 2. Before changing any code, send me a short plan: the target folder layout,
>    the order of the work, and any doubt or risk. Wait for my approval.
> 3. Work in small steps. After each step run fmt, clippy and all tests and give
>    me a one-line result. If something fails, stop and tell me; never weaken or
>    skip a test to make it pass.
> 4. Prefer small, clear changes that I can explain in an interview. No heavy
>    dependencies, and nothing that is not justified by the documents.
> 5. Ask at most one question at a time, and only if you are blocked.
>
> ## What to do
>
> A. Improve the project in the areas described in docs/IMPROVEMENTS.md:
> correctness, safety and robustness, efficiency, and maintainability.
> Do correctness and safety first, then efficiency, then presentation.
> Adopt a performance change only if a measurement shows a real gain.
>
> B. Reorganize the project so the layers and the dependency direction are
> visible: domain logic separate from input and output, documents, scripts
> and tests in their own folders. Do moves and renames in separate steps from
> behavior changes, and keep every path in README, Makefile, Dockerfile and CI
> correct.
>
> C. For every modification, be able to say why it was made: the problem it
> solves, the benefit, and the trade-off. Keep the written record current as
> you go: reflect the work in TASKS.md (tick finished tasks, add any new ones),
> in the improvement and architecture documents, and in the README, so the
> documentation always matches the code.
>
> ## Quality gates (all must pass at the end)
>
> - build, tests (unit, integration, property, doc-tests), fmt and clippy clean
> - the example from the spec still produces exactly the expected output
> - the Docker image builds and runs against a fixture (say so if Docker is
>   unavailable)
> - no new way for input data to cause a panic or unsafe behavior
>
> ## Final report
>
> 1. A table of every change: area, what, why, and the test or measurement that
>    proves it.
> 2. Before and after numbers for any performance change, or "no change adopted".
> 3. The final folder tree.
> 4. Anything you did differently from this prompt, and why.
> 5. The assumptions I should be ready to defend.
> 6. An entry for docs/PROMPTS.md, and which of your suggestions I should mark as
>    accepted, changed or rejected.

## Decisions that came from prompts

- Money type: `rust_decimal` instead of hand-rolled `i64` fixed point (less custom parsing code, proven, checked arithmetic). Delegated in P2.
- Disputes apply to deposits only; cross-client references are rejected (documented in README).
- Locked accounts reject new deposits and withdrawals.
- Diagnostics go to stderr, never stdout.

## AI suggestion review

| Suggestion | Source | Decision | Why |
| --- | --- | --- | --- |
| `rust_decimal` behind an `Amount` newtype | P2 | accepted | Exact base-10 maths, checked operations |
| Only deposits are disputable | P1, P2 | accepted | Disputing a withdrawal would add funds |
| Locked account rejects new deposits | P1, P2 | accepted | Bank-like freeze; documented assumption |
| Property tests (`proptest`) | P1 | accepted | Checks `total == available + held` |
| Do not track duplicate withdrawal ids | P2 | accepted | Avoids up to ~4B set entries; documented trade-off |
| Make `Account::total()` explicit with `Option<Amount>` | P6 | accepted | Eliminates silent zero fallback on arithmetic overflow |
| Centralize locked account check in `ensure_unlocked` | P6 | accepted | Isolates the business policy to a single, maintainable method |
| Add `#![forbid(unsafe_code)]` and doc-tests | P6 | accepted | Compiler-enforced safety guarantees and verified examples |
| Naive ledger reference-model property test | P6 | accepted | Proves state correctness across random transaction sequences |
| Parser robustness test with arbitrary bytes and fuzzing | P6 | accepted | Verifies no panics on corrupt, hostile, or malformed inputs |
| Aggregate skipped rows by category with stderr summary | P6 | accepted | Eliminates per-row stderr I/O bottlenecks on large datasets |
| 128KB `BufReader` on input stream | P6 | accepted | Demonstrated ~13% reduction in execution time |
| Separate `src/domain/` from `src/io/` | P6 | accepted | Enforces strict dependency direction (domain has zero I/O imports) |

Edit this table if you changed or rejected anything.

## Template for new entries

```
| Pn | <what was asked> | <what it produced> |
```

Record, for each AI suggestion: accepted / changed / rejected, and why.
