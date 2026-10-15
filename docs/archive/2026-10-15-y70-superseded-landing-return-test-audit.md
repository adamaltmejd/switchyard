# Y-70 superseded landing return

Review3341 found that a supported producer continuation before landing
mutation intent can invalidate the choice and approval while merge-tree
is running. The conflict return then resolved the producer from the current
missing choice, staged a bundle for the default writer, and installed an
obsolete repair because its final guard checked only ticket edits.

The queue now shares the store-backed approval supersession reading with
ordinary landing eligibility: active approval, live attempt, current exact
choice and candidate, proof, ticket/edit and plan revision. The return also
checks the landing row against its bound approval. An obsolete return ends
withdrawn before staging and leaves the current continuation alone. A valid
return resolves its producer from the immutable choice bound by that
approval. It builds the bundle under the existing landing directory, then
revalidates in the transaction before delivering it to that producer and
recording repair. The canonical lock excludes supported workspace starts
and sync during bundle delivery; the final transaction also fences store
changes such as ticket edits. Temporary landing files never become worker
input after supersession. Reservations, host ownership, target locks,
prepared evidence, exact refs and the open-intent fence keep their existing
lifecycles and guards. No new lifecycle or compatibility path was added.

Read together: initial preparation admission and approval holds; all
conflict, automatic red and explicit merged-failure repair return callers;
choice invalidation and supported workspace continuation; producer lookup,
isolated input staging, frozen feedback and target-fetch prompt; final
return transaction; subsequent selected scheduling, approval and CAS intent.
ARCHITECTURE.md records the implemented return fence and G10 evidence.

Disposition: extend the existing five-profile G2/G8/G9/G10 real-host
scenario, retaining the same frozen reviewer and read-only picker. Its
selected B has already repaired a real candidate-gate failure and received
its own blocking review at the configured limit. Exact override approval
still binds only that source's failed review and required gate. A fixture
Git wrapper holds the real merge-tree invocation before intent against an
externally advanced conflicting target. The supported B workspace start
records a continuation and withdraws that approval while the merge stays
held. Launch takes the same target lock, so the host releases the merge
before waiting on B's next model turn. No sleep or binary barrier is used.

The store and host files prove that the obsolete landing ends withdrawn,
its exact approval stays withdrawn, and the attempt's next, target, red
count and cleared choice remain unchanged. The continuation's source,
settings, frozen inputs and revisions remain unchanged. Neither A nor B
receives a target bundle, and no bundle is staged for the obsolete landing.
The existing fresh-conflict control now follows that continuation: B
publishes a new immutable candidate, receives its own review, choice,
passing required check and exact approval, then conflicts validly. Only
B's repair fetches the actual target bundle and resolves it. Fresh own
judgments, omission acceptance and exact approval precede landing. The
original post-intent continuation refusal remains unchanged.

Expected producer count, settings and contents come from the five fixture
scripts. Git in the actual producer clone supplies new heads, fetched
target and merge ancestry. Store comparisons establish identity agreement
and absence of obsolete mutation, not an expected verdict copied from
Yard. The comment names sabotage: use the missing current choice fallback
or fence only on ticket revision. Existing conflict coverage lacked a
source continuation while merge preparation was held; the existing
post-intent refusal exercises a different boundary. This is the smallest
coherent extension of the same valid-conflict repair scenario.

No original control or assertion was dropped. Changed fixture version
numbers reflect one extra B publication; the final content remains B's.
Approval history now explicitly retains the withdrawn override, retired
fresh-conflict approval and landed final approval. The exact five-producer
count, original model/reviewer controls, failure policy scenario, stale and
foreign choices, immutable history, independent bases and empty-panel
policy scenario remain intact. Test lines replaced: 14. Binary lines
replaced: 25, including extraction of shared approval identity guards.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings, and cargo build --locked passed. Native/e2e was not run in
this box, as instructed. Host qualification remains operator-owned after
review. No .yard project configuration changed.
