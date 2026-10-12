# Y-92 completed-cleanup ownership repair, 2026-10-11

## Exact-head diagnosis

The operator's clean full 72 host suite at
0325ee562a485abc72e5e53229f6395bebbfd695 ended 71 passed, 1 failed,
exit 101, in 904.73 seconds. Review 927 reported NONE. Focused gate-error
(32.43 seconds), publication recovery (18.13 seconds), and both package
cases (369.77 seconds) passed. The full run also passed publication recovery
and both package cases. These results do not authorize landing 0325 or
establish acceptance of this changed head.

a_gate_errors_start_reruns_that_gate failed before child admission, in
start_selected at line 70. Trusted MCP yard_attempt_start with default
workflow overrides returned refused, input=plan, reason=active_plan.
The helper had observed planner cleanup end and checked reservation release.
Before fixture Drop, read-only external execution, audit and attention
snapshots showed planner 1 completed unchanged with its real report,
teardown=1 and reserved=0. Cleanup 2 ended removed at seq 13,
2026-10-11T07:35:59.966Z. Its release probe raised ownership attention 1
at seq 14, 07:36:29.971Z. A later genuine checked release emitted seq 15
at 07:36:43.078Z and reserved=0. That exact ownership item remained open.
No new attempt or landing child existed.

capacity::reap calls release and does nothing further for an already-ended
cleanup. release cleared the reservation after proof but did not resolve
that obsolete item. Selected admission correctly refused the remaining
question. The admission guard is preserved; the test does not answer it.

The improved kernel observer separately verified pinfold 175315 and podman
175316 working directory and ancestry for the gate-error fixture. Their
ages spanned 10 to 29 seconds across the 30-second failed probe. Sampled
threads showed futex waits, not a confirmed fcntl waiter; image lock owner
175343 was present. A subsequent same-fixture podman 175893 listing at
ages 10 to 12 seconds had fcntl_setlk and a kernel image-lock waiter behind
175343, then genuine release followed. This ties delayed listing to this
fixture. It does not establish the first internal wait's precise cause or
the historical 1751 or Y-95 root cause. The operator retains the terminal
log, live snapshots and process ancestry locally. No upstream report was
published. This repair makes no external runtime-delay claim.

## Bounded production repair

Only a supported cleanup currently ended with outcome removed can resolve
its own red:ownership item. The existing release transaction rereads that
exact execution after current pinfold box absence and host-lock proof. It
releases capacity and resolves the exact kind/reason/execution match as
reconciled in the same transaction. It does not create a result, judgment,
teardown, execution, publication or new eligibility truth. Failed/incomplete
cleanup, unsupported lock contracts, unrelated questions and every other
execution kind retain their existing behavior and operator exits.

Cleanup removes the attempt directory, including its former effects lock
path. Its existing execution lock now lives at the project-local effect
directory for that execution, outside the removal payload. Its descriptor
still follows the same host-child inheritance contract. The cleanup probe
requires that current lock file to exist before using the existing lock_free
observer. Directory absence or a deleted inode is not proof. The directory
is retired with the existing removal function after the successful release
transaction. A missing current cleanup lock remains unknown, including an
older cleanup that deleted its lock. No replacement lock or legacy proof
is manufactured. No lifecycle, owner, lease, new dependency, migration or
daemon selection path is added. Binding host-lock, capacity and G5/G8
clauses are updated together.

The exact unstarted-child repair and prompt-independent published-review
owner selector from 0325 remain unchanged. Package, CI, pins, sealed closure,
native bridge and Y-97/Y-98 controls remain unchanged.

## Test-audit Gate

Keep and extend a_gate_errors_start_reruns_that_gate before its existing
selected admission. Preserve every original gate-error assertion and the
unstarted-child extension. No new test entry is added; the suite remains
72 tests. Extract only the existing trusted selection call so the same
real MCP operation supplies refusal and allowed controls.

1. G5/G8 now name completed-cleanup ownership reconciliation after a real
   listing executable failure and restored real listing with the current
   lock held. They require the exact reservation and question before proof,
   then the original completed cleanup and planner result with a new
   selected admission, without an operator retry or replayed cleanup.
2. Sabotage: omit the exact completed-cleanup resolution from checked
   release. The named resolution wait and allowed selected admission fail.
   Release without current host proof: the held-lock retained-state and
   admission-refusal blocks fail. Keep the lock in the removed attempt:
   reopening the surviving current lock fails. The test comment names these
   sabotage changes; no timeout is a passing observation.
3. Existing G5 child/review recovery holds a live inherited owner and
   settles an interrupted execution. It cannot show an already-successful
   ended cleanup blocking admission after its capacity release. The earlier
   gate-error extension loses a child before controller start; it cannot
   expose this planner-cleanup boundary. This extension forces the measured
   failure in the existing real planner and successful selected-gate chain,
   without another scenario or dependence on image-store overload.
4. The fixture wraps real pinfold commands. A fixture-owned executable link
   to installed pinfold supplies actual listings with unchanged arguments,
   output and exit status. Removing only that link produces a real shell
   executable-availability failure. Restoring it runs the actual tool; no
   fake list output or success is supplied. Each real down completes, then
   its wrapper holds the reply on a fixture FIFO. The fixture releases the
   planner's own teardown and holds the cleanup's actual down completion,
   identified by read-only external execution rows and the real box argument.

Before releasing cleanup, the external current lock observer shows a held
lock, and the store shows that cleanup running/reserved with the supported
contract. After removing the listing link and returning the actual down
result, status history names the same cleanup's removed end and ownership
refusal. The removed attempt directory is observed from the host. The test
reopens the surviving current lock read-only after that removal and acquires
a real external lock; it creates no private path and locks no deleted inode.
The real trusted selection call must refuse by code and plan/active_plan
identity, and the external store shows only the original attempt.

Restored real listings hold successful replies on another FIFO. One reply
is returned while the host lock remains held. The next actual listing call
proves the preceding probe finished without releasing capacity or resolving
the exact question. Store and MCP observations retain that refusal. These
are handshakes for distinct automatic reconciliation calls, not test retries.
The fixture then unlocks and returns the held real listing. History must
name checked release and reconciled resolution of that same execution/item.
The cleanup stays ended/removed with null result/teardown, without replay;
the exact completed planner execution is unchanged. Fixture audit content
comes from publish_report, and the retained planner report, teardown and
reservation are observed through supported attempt show. The same selected
MCP operation now succeeds with no operator question answer. The complete
original gate-error and landing controls follow unchanged.

Expected lock ownership comes from the OS. Expected execution/item identities
come from external read-only store rows, and must agree with real history.
Removed/retained/reconciled expectations come from the binding G5/G8 clauses.
MCP reasons come from the spec-named admission guard. Before/after planner
agreement tests preservation, not a copy of Yard logic. FIFO zero status is
the actual delegated command's status, not a mocked response. No assertion
is deleted or weakened. No sleep, clock wait, conditional retry, deadline or
thread-cap change, runtime prune/kill, host installation, private launch or
store write, binary hook, public endpoint or model/harness substitution is
introduced. No original recovery or package control is removed.
Diff counts: 4 test setup/call lines removed, 207 added; 5 production lines
removed, 29 added across capacity and cleanup. Removed assertion blocks: zero.

## Validation and remaining gates

Candidate cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings,
cargo build --locked and git diff --check passed. Source comparison confirmed
all 15 guarantee rows remain and rows outside G5/G8 are byte-identical.
No e2e or boxed/image/host package execution is performed here.
Old archives and operator .yard, daemon,
installed binary, model and reviewer remain unchanged. No push, publication
or new active ticket is made.

Fresh changed exact-head source/test audit, focused recovered-cleanup/admission,
existing gate-error and publication controls, both package cases, full 72 host
suite, published review and successful actual landing remain required.
If external runtime still blocks validation, report its concrete evidence;
do not retry an unchanged head or weaken checks. Review 923's generic
missing-path P3 diagnostic remains uncorrected despite review 927's NONE.
Y-93, parent Y-64 and release through Y-80 remain required; Y-81 stays parked.
