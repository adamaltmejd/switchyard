# Y144 selected failure disposition correction

Operator gate2795 on `06c3e59` passed 78/79 scenarios in 567.39 seconds.
G2 passed with real image consumers after both optional metadata corrections.
G3 stopped at its selected failure's attention kind: actual `candidate`,
expected `red`. The full raw result remains in
`approved-y144-combined-repair-gate-capture`.
This failed candidate is not qualified. Earlier archives remain unchanged,
including their red-attention claim. Gate2770, gate2780 and the surviving
`b063511` diagnostic remain failed or unqualified evidence.

## Protocol trace

ARCHITECTURE.md's selected eligible failed landing gate contract names
`candidate:landing`. The fixture explicitly selects `operator_failure: true`.
`jobs/queue.rs::land` records the failed check, obtains `failed_evidence`,
raises that exact kind and reason, then ends the parent as failed. This is
the existing exact acceptance route. Correct only the kind expectation.

`failed_evidence` serializes the check's execution, check ID, name, verdict,
head and plan revision, the failed result, node and consumed definition.
Its landing identity contains execution, approval, target, base, head, proof,
ticket revision, plan revision and prepared input. `config::Gate` serializes
name, command, stage, runs_in, timeout_minutes and env; the current fixture's
complete literal expectation matches those fields without optional metadata.

The public MCP `attempt.accept` schema requires project, ticket, attention,
plan_revision, target, head and text. `jobs/admit.rs::candidate_decision`
also requires check for a failed result and compares landing, child, node,
base, revision, old and merged against this attention's evidence. The test
supplies all those bindings and proof from the actual attention. It waits
for checked child and parent release before accepting. The handler checks
canonical and prepared refs, active approval and current eligible evidence,
then resolves the exact attention with nonempty text. Its return is
`{attention, accepted, evidence}`. MCP transports the daemon response in
structuredContent, which the fixture returns; `result.accepted == true`
is the existing supported success observation. No request or result shape
needs correction.

## Test-audit Gate

| Gate question | Answer |
| --- | --- |
| Guarantee and clause | G3's adopted selected landing definition and exact prepared merged-ref evidence before failure acceptance. The existing Shown by clause remains accurate. |
| Named sabotage | `original-landing-decision-evidence` must fail the direct committed selected gate-definition assertion, not a deadline. Both independent intent-guard faults remain required. The existing test comments name these mutations. |
| Distinct hardest case | Explicit adoption adds a gate absent from original admission. Its real failure exposes the consumed definition before exact acceptance. Initial-definition failures cannot expose reporting from the wrong admission after adoption. This corrects disposition in that same scenario and adds no twin. |
| Outside expected values | The architecture supplies candidate:landing disposition. Committed fixture literals supply the complete gate definition and command. Host Git supplies prepared HEAD. Actual check/child/revision evidence binds acceptance; store snapshots establish checked release and immutable history. |

Static Gate accepts this one-line fixture disposition correction. Test lines
removed: 1, replaced by the correct kind. Binary lines removed: 0. No
assertion block is removed. Reason, failed result, selected definition,
checked release, exact acceptance, historical snapshots, open and resolved
intent no-write controls, canonical retirement and successful requeue remain.
G2's implementation-digest correction and real ENV/COPY, Git source/context,
unchanged host-command and historical controls remain unchanged.

`cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`
passed. This is a bounded Gate, not a sweep. Host e2e and named independent runtime
faults were not run in this box, as instructed. The full native suite,
original-landing-decision-evidence fault, both intent guards, other named
faults and rebuilt restored G2/G3 positives remain post-approval gates.
All original Y144 acceptance and parent and packaged integration obligations
remain mandatory. No binary, .yard, guarantee or lifecycle change is made.
