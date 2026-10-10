# Y-84 default-plan equivalence repair — 2026-10-10

Operator source review of bdb8e63 found that the inspected plan omitted the
landing execution of candidate-stage gates and described all checks as
reading the implementer's candidate. This corrects the default-chain
disposition in `2026-10-10-y84-test-audit.md`. That original evidence stopped
at approval; it did not establish full native equivalence. The earlier
archive remains a record of that review.

`jobs/queue.rs::land` iterates the entire configured gate set and records
each child with reason `landing` and head `merged`. Candidate checks filter
by configured stage and run before review. Both boxed and host gates have
private writable checkouts. No native gate behavior was changed.

The translation now names candidate and landing gate instances separately.
Candidate gates and reviewers reference `implementer.result`. Landing gates
reference the existing typed external input `merged`, with type `candidate`.
The native queue supplies that input. Gate nodes describe their writable
checkouts using the existing access axis. There is no merge node, new
lifecycle, graph runtime, admission operation or active-plan freeze.

## Test-audit gate

The changed block is in `g06_plan_inspection.rs`. The original structural
refusal, target identity and no-admission blocks are retained.

1. G6's revised shown-by clause requires the literal default plan to
   describe candidate-stage checks before review and every configured gate
   at landing, with their distinct inputs. The native control now lands.
2. Sabotage: filter landing plan nodes by declared stage, or use
   `implementer.result` as their input. Either differs from the literal
   fixture. Skipping the native second candidate-gate execution or running
   its landing checkout on the candidate fails the native input observation.
   The test comment names the plan sabotages.
3. The fixture has one candidate-stage gate and one landing-stage gate.
   Advancing the target after approval attention but before answering it
   makes the merged head distinct from the candidate. G13's existing host
   gate checks remain the survivor for host placement and proof semantics;
   none compares an inspected plan's full gate instances with native boxed
   candidate and landing inputs. The old generic feature-content assertion
   is removed; G7 remains the survivor for landed worker content.
4. The expected graph and three check instances are literal fixture data,
   from the binding native phase contract. The candidate head comes from
   host git in the attempt clone. The merged head comes from host canonical
   git after landing. The externally read execution rows and boxed git
   command output observe each actual check input. No expected plan is
   reconstructed from Yard output or resolver logic.

Inspection still precedes the worker control. Read-only store observations
verify no ticket, attempt, execution or audit mutation during resolution.
No sleeps, public endpoints, mocks or binary barriers were added.

Candidate validation: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings`. Host e2e execution is
reserved for the landing gate, as instructed by the ticket.

Diff evidence: 16 test lines and 6 binary lines replaced or removed. The
removed content assertion has G7 coverage; no guarantee clause was weakened.
