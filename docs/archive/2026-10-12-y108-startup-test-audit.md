# Y-108 frozen startup test audit

The operator reports that candidate
b701196ca98d7e91d71feddb6f26c8a69d930da3 passed all five unchanged focused
host controls: selected read-only revision, ticket-edit report, quiescent
revision, live review teardown and held implementation/edit. These successes
stand alongside the earlier 571a3f839920e59e0f2d000d8222a2cdefab5b70 read-only
panic failure recorded in 2026-10-12-y108-read-only-resolution-repair.md.
Neither record claims full-suite validation.

The operator's startup-only mutation changed just the first
`attempts::for_execution(conn, &row)` in `supervise::implement` to
`attempts::get(conn, row.attempt)`. With every other reader and assertion
unchanged, the held implementation/edit scenario passed in 16.194 seconds
and the selected read-only scenario passed in 7.112 seconds, both exit 0.
Their model holds happened after startup had consumed the admission.
Frozen store rows and successor diffs did not expose the wrong startup
reader. This is a coverage failure, not an unchanged-candidate product bug.
The operator restored source exactly, with a successful restored test build
and empty tracked diff.

Evidence supplied by the operator is under
`/home/yard/validation/y108-571a3f-review-20261012`, including
`operator-startup-latest-plan-sabotage.patch`, the corresponding case logs
and `operator-startup-latest-plan-sabotage-result.json`. The host evidence
is not independently replayed in this box.

## Gate disposition

Extend `an_edit_reaches_the_next_prompt_and_stop_delivers_it_sooner`, rather
than add a twin. G9's shown-by clause now names startup while a real capacity
owner holds the only execution slot. G2's judgment identity clause and its
held review scenario remain unchanged. The read-only scenario also remains
unchanged. Existing ticket-edit, explicit-stop, head and gate assertions
remain; implementation queries select the target ticket because the new
capacity owner has its own execution and attempt.

The parked target ticket preserves its fixture identity Y-1. A second
ticket's real Pi worker holds a host-model reply at capacity one with two
admission lanes. Unparking Y-1 admits its running execution, whose
`execution.waiting` event names reason capacity. Store inspection proves
original plan revision 1 and reserved 0 before revision CAS accepts 2.
No stop or new start is issued for Y-1. Capacity reservation precedes the
first poll of implement, so the admitted profile changes before the startup
reader. Releasing the owner allows that same execution's initial prompt to
reach the host model. It must contain the fixture's original instructions
and omit its revised instructions. Its model reply then holds for the
existing ticket-body edit, candidate finish and successor assertions.

The precise startup-only mutation above makes the initial prompt assertion
fail directly: it selects revised instructions instead of the original.
The previous scenarios could not observe this path because startup had
already happened. A capacity waiter is the deterministic external point;
there is no binary barrier, sleep, retry, mock or direct store write.
Expected instruction strings come from the fixture configuration. Ticket
identity comes from fixture CLI creation, execution identity from the audit
event and frozen inputs from read-only store observation. The host model
request observes consumed instructions independently of those stored rows.
The accepted CAS is the positive control. The existing successor prompt
must still carry revised instructions and the labelled operator diff, and
host Git and gate rows must still name the resulting candidate. Existing
held-review and unknown-owner controls supply checked teardown evidence;
this change adds no duplicate ownership scenario.

No assertion promise or binary line is removed. Query replacements exclude
only the new owner ticket; the same target execution fields and reason
sequence remain asserted. cargo fmt --check and
cargo clippy --all-targets --locked -- -D warnings both passed.
E2e is prohibited in this box and remains the host gate.
The extended unchanged scenario must pass on the host, then the precise
startup-only mutation must fail at the original-instructions assertion.
Those new host results remain pending; this audit does not claim them.
