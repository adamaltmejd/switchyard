# Y-104 failed-parent release observer repair

Assessment base: `2cdabf24e78534ba06ba25745764dc4a40423712`.
This is a scoped test-audit gate for G8's
selected_failures_continue_without_rewriting_judgments scenario.
The prior integration archive remains unchanged.

## Actual operator failure

The operator ran the complete unchanged focused G8 scenario on that exact
candidate against the real host runtime. Cargo exited 101. One scenario
failed in 81.33 seconds; elapsed time including build was 96.0582 seconds.
The new explicit merged repair assertion expected result.accepted false but
observed Null. The response was stale by ownership: the exact failed parent
was ended with outcome fail but still reserved, against expected released:true.
The daemon's ownership refusal was correct. The failed result is preserved.

The repair-route event observer, bound-feedback assertions, retired approval,
unchanged failed rows, held no-mutation/intent observations and fresh fully
verified/approved repaired landing were unreached. This run establishes no
successful explicit merged repair or feedback-drop sabotage.

## Observer change

Watch::find searches cached events. Matching only the original failed parent
identity selected its earlier gate-phase capacity release. That release was
not its final checked release after failure.

The repaired observer follows the existing merged acceptance control:
first find the exact parent's execution.ended with outcome fail strictly after
the landing question, then its execution.released strictly after that end.
Both bounds come from the actual history sequence and exact execution identity.
No production code or ownership guard changed. The observer does not release
capacity, retry the decision or substitute its reading for daemon proof.

## Assertion gate

Disposition: repair the owning existing observer. No assertion or guarantee
clause was removed or weakened. G2/G3/G8 Shown by already name complete failed
evidence, no consumption of the failed input and the successful explicit
selected merged repair path; their contract stays unchanged.

1. G8's explicit selected merged repair clause owns this control. G3 requires
   checked ownership before continuation; G2 retains truthful original rows.
2. The existing test comment names feedback-drop sabotage. Replacing
   next.get("feedback") with None in advance leaves repair runnable but fails
   the complete bound feedback assertions. No new sabotage claim is made.
3. The earlier acceptance block has the correct post-end release observer,
   but its answer is accept. It cannot establish the distinct repair decision,
   retired approval and repair feedback. This is the same harder merged-input
   case, with a target advance and a real held repair turn. No new scenario
   or duplicate assertion block was introduced.
4. The history CLI exposes event identities and order from outside the binary.
   The fixture failed gate supplies fail; the ownership contract supplies the
   required post-end release. Host Git still supplies old/merged/fresh heads,
   and the unchanged fixture and store observers supply the original failure,
   carried feedback, retired approval and fresh selected verification results.

All earlier acceptance, restart, stale/no-write, ownership, skipped/empty and
infrastructure controls remain. The repair-versus-incorrect-landing event
observer still runs before waiting on the real repair latch. The complete
feedback, original failed snapshots, no canonical mutation or intent while
held and successful fresh candidate/landing assertions are unchanged.
No sleep, retry, deadline/concurrency change, mocked runtime or binary barrier
was added. No restart point was changed. Test lines removed: 2 observer lines.
Binary lines removed: 0.

## Validation and retained gates

Worker `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. Clippy compiled
the changed e2e target; it does not establish runtime success. Worker host
e2e and sabotage remain unrun. No .yard or operational runtime/configuration
change was made. No subsequent runtime failure is inferred from compilation.

The operator must rerun the exact repaired complete control and meaningful
feedback-drop sabotage before approval. Published review and actual merged
host gates remain required. The initial production feedback binding remains
under review. Y-104, Y-96 and Y-61 retain their full integration contracts;
Y-80 release scope is unchanged and Y-81 remains parked. No parent completion
is claimed.
