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

## Prompt text

### P1
Attachments: the take-home spec (PDF) and the recruiter email with the AI
policy. Neither is reproduced here; the spec file must not be committed.

> I need to implement this, help i need all documentation, workflow for ai validate, test, if posible docker or similar to running it, i have 3 days to implement. Following all the rules present on the documentations.

AI response: plan only, no code, requested review before implementation.

### P2
> Choose the best option, if there any trick on the coding test, also give me all the archutecture used in the solution, also the prompts we can map in a documented way too

AI response: picked `rust_decimal`; listed the traps in the spec (whitespace in
headers, optional amount, stdout purity, cross-client disputes, negative
available after a dispute, locked-account ambiguity, scale hint); built the
implementation, tests, scripts, Docker, CI and docs.

### P3
> can u give me a graph of the architecture?

### P4
> a architecture draw (more high level) so i candraw it on my notepad? like the system design?

### P5
> give me all the prompts for the project, attached part of the documentations, so i can add a complete prompt log

Follow-up refinement:

> not need the commiit flow, change my name commit... just the thing related to the project, and also the complete plan implementation, in separated markdown

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

Edit this table if you changed or rejected anything.

## Template for new entries

```
| Pn | <what was asked> | <what it produced> |
```

Record, for each AI suggestion: accepted / changed / rejected, and why.
