# Improvements

Changes made after the first working version, and why. Each one targets a
scoring area of the coding test and is verified by a test or a measurement.
Behavior seen from the CLI (input, output, exit codes) does not change.

| ID | Improvement | Why (problem) | Benefit and scoring area | Trade-off | Verified by |
| --- | --- | --- | --- | --- | --- |
| I1 | Remove the silent fallback in `Account::total()` | `unwrap_or(ZERO)` would hide an overflow bug and print a wrong total | Correctness, safety: errors are explicit, no silent wrong numbers | Slightly more code at the call sites | Overflow unit test |
| I2 | Use the stats returned by `process` and print a summary on stderr | Counters were computed but never used, which is dead code | Maintainability, observability: a user sees how many rows were applied or skipped | One extra stderr line | CLI test on stderr |
| I3 | Aggregate skipped rows by error kind instead of one log line per row | On millions of rows the per-row logging dominated run time | Efficiency: lower I/O cost, readable summary | Less detail per row (detail kept for small inputs) | Timing on a large file |
| I4 | `forbid(unsafe_code)`, `warn(missing_docs)`, rustdoc examples | The safety claim was only in prose; public items were undocumented | Safety, maintainability: the compiler enforces the claim; examples run as doc-tests | None meaningful | Build and doc-tests |
| I5 | Test `process` directly with in-memory input, plus output format edge cases | The "works on any reader" claim was only tested through the binary | Correctness: proves the engine is reusable (files, sockets) | None | New tests |
| I6 | Reference-model property test | Invariants alone do not prove the results are right | Correctness: a naive replay model catches logic errors invariants miss | Slower test run | Property test |
| I7 | Parser robustness test with random input | "No panics on bad input" was a claim, not a check | Safety and robustness: evidence for the claim | Slower test run | Robustness test |
| I8 | Measured efficiency work: buffered input, hasher, compact stored deposit | Memory grows with distinct deposits; unmeasured guesses are risky | Efficiency: adopted only if the numbers improve | A new dependency if a faster hasher wins | Before and after numbers in the README |
| I9 | Locked-account rule in one function | The spec does not say what a locked account does on later deposits | Correctness risk control: flipping the assumption is a one-line change | None | Existing lock tests |
| I10 | Dependency audit in CI | The test allows "common crates you deem secure"; that should be checked | Safety: known advisories fail the build | CI takes longer | CI run |
| I11 | "Limitations" section in the README | Some limits (duplicate withdrawal ids, memory growth) were not stated in one place | Honesty and completeness: reviewers value known limits | None | Review |
| I12 | Project reorganization: `domain/` and `io/` modules, separate `docs/`, `scripts/`, `tests/` | The flat `src/` hid the dependency direction | Maintainability: layers are visible, `domain` cannot depend on `io` | Moved files, updated links | All gates still pass |
| I13 | Documentation kept in sync (README, architecture, plan, tasks, prompt log) | Docs drifted from code after changes | Basics and maintainability: a reviewer can trust the docs | Extra review time | Link and path check |

## Principles behind the choices

- Fix correctness and safety issues first, then efficiency, then presentation.
- Do not change behavior the grader can observe.
- Measure before adopting any performance change.
- Prefer small, explainable changes the author can defend in an interview.
- Keep moves and renames separate from behavior changes.
