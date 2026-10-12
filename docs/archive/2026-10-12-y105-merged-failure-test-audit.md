# Y-105 selected merged failure acceptance and test audit

Source: 51a6078, including the actually landed Y-103 implementation and
722ad5a host validation record. This is the first Y-104 implementation child.
Y-104, Y-96 and Y-61 remain incomplete until their full original contracts
have exact-head operator host evidence.

## Change

Retained landing gates may select operator failure handling before admission.
An eligible completed failure records an ordinary failed gate check and raises
candidate:landing. Approval stays active and later approvals on that target
wait for its answer. Resolved candidate attention remains
the only accept/repair decision store. Candidate current-check readings exclude
landing children, including when candidate and merged heads coincide.

Merged evidence binds the original preparation, candidate and proof, target,
old and merged heads, admitted revisions, approval, selected node, child,
check, inputs and complete result. Answers additionally name these identities.
They probe canonical and the prepared ref and prove predecessor ownership gone.
Consumption uses Y-103's predecessor ownership proof, rereads failed evidence
and the exact acceptance, and preserves the shared final CAS. Repair invokes
the existing returned path. Failures remain failed; a fresh preparation cannot
inherit an answer. No migration, decision store, worker grant or dependency
was added.

## Assertion gate

This is a scoped gate, not a whole-suite sweep. No existing guarantee clause
was removed. The changed observers replace or extend existing scenarios.

| Disposition | Guarantee and clause | Sabotage | Outside observer and expected value |
|---|---|---|---|
| Extend g01 reduced selection control | G1/G3/G8: retained landing selection remains queue-authorized after a full default chain | Reject operator_failure on a retained landing node | Real operator admission and canonical Git; the reduced graph is a fixture literal. Existing node and omitted-result observations remain. |
| Extend g03 unchanged prepared restart | G2/G3: accepted failed evidence survives a held remaining child and lands its pinned input | Rewrite failure to pass, replay completed evidence, consume before teardown, regenerate merged input or bypass shared CAS | Complete store check/result/input snapshots, host Git pinned ref, recorded host process group and inherited lock. Fixture false supplies fail/failed; host Git supplies old and merged heads. |
| Extend g03 changed-target restart | G2/G3: changed preparation carries no old acceptance; a delayed answer is stale without writes; a fresh answer succeeds | Inherit acceptance after target advance or omit old-target comparison | Host commit-tree/update-ref advances canonical while the real remaining child is held. Store and audit snapshots bracket the delayed call. Its exact expected/current heads come from host Git. Final selected children exclude original evidence. |
| Extend g08 selected failures | G2/G8: default repair precedes candidate and merged acceptance; remaining selected checks run and failed records survive reopening | Return selected failure automatically, rewrite it, stop at acceptance or accept ended-attempt evidence | Real CLI/MCP, canonical Git and complete failed check/result/input snapshots. Fixture gate command supplies failure and remaining success. A late duplicate after cleanup/restart names attempt 3 versus no live attempt and writes no audit. The original successful answer is its control. |
| Extend g08 infrastructure retry | G8: merged infrastructure errors cannot be accepted and do not route repair | Publish an eligible failed judgment on real box-up error | Fixture pinfold wrapper refuses actual box up. Store has no judgment/output/check; acceptance returns stale by attention with no audit. Removing the wrapper refusal permits the real same-prepared-input retry and landing. |

These are the hardest existing edges: a selected retained node after omission,
a restart with an actually held remaining host child, target advance while
that child is held, a delayed duplicate after fresh preparation or cleanup,
and a real infrastructure error followed by ownership reconciliation and retry.
Candidate acceptance blocks do not cover merged inputs or retained approval.
The original passing-child and interrupted-child observations remain in g03;
there is no new parallel scenario with the same mechanism and observation.
G3's existing held final Git owner and final-CAS observers remain the shared
ownership and CAS controls.

Before restart g03 proves the pinned ref, prepared row, completed passing
child, failed snapshot, absent canonical intent, live host process group and
held inherited child lock. The gate's FIFO holds that point until the daemon
is killed. No binary barrier, sleep, retry or public endpoint was added.

Diff accounting before this archive: test lines removed 19; binary lines
removed 28. These are edits to existing scenarios and paths, not deleted
assertion blocks or weakened guarantees.

## Validation and operator gate

Worker checks passed: cargo fmt --check and
cargo clippy --all-targets --locked -- -D warnings. Clippy compiles the changed
e2e targets. The worker ran no e2e suite or sabotage, as the ticket explicitly
reserves the real container runtime for the operator's approved landing gate.
No development daemon, shared socket/runtime or .yard change was made.

The operator still owns exact candidate-head host e2e and sabotage evidence.
Required sabotage includes failure rewritten to pass, acceptance inherited on
changed input, consumption before checked teardown and bypassed final CAS.
Compilation is not host execution evidence. This record does not close the
parent contracts or claim their remaining landing-omission acceptance scope.
