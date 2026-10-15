# Y-70 named reader admission

The operator's source preflight found the selected gate admission race in
named reviewers too. Writer scheduling observed an eligible producer and
candidate outside the reader-start transaction. A replacement publication
could make execution binding resolve a new producer while checkout columns
still named the observed candidate.

The existing reader-start transaction now requires the observed plan and
choice, a live attempt, no pending continuation and no superseding ticket
edit. It rereads the observed producer execution and requires its result
to remain eligible and unchanged. Current declared binding must name that
exact execution and the full observed candidate value. Its base, head and
proof must also match the reader's execution columns before row creation.
Unavailable or obsolete input returns without a reviewer execution or spawn.
The normal execution creation uses the same Start and binding function in
that transaction. This applies to candidate-only and selector writer graphs.

Read together: named producer selection, execution result eligibility,
current graph binding, reviewer start and detached checkout, selected gate
start, candidate publication and automatic gate repair. This read also found
that 2689c74's selected gate comparison was too strict: writer candidate
values include candidate, workspace and producer metadata in addition to
base, head and proof. Comparing the full value to a three-field object
would refuse valid admission. The gate now compares the three identity
fields individually, preserving all provenance in its immutable input.
The earlier archive records the preceding change; this record corrects its
claim about valid selected admission.

No new lifecycle, global identity framework or alternate source lookup was
introduced. No assertions changed. Existing five-producer same-reviewer
exact-input/source-change and G4 named-writer controls remain intact. There
is no demonstrated external held point inside this admission race, so no
timing race, sleep, retry or binary barrier was manufactured.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings and cargo build --locked passed. Native/e2e was not run as
instructed. The affected
normal reviewed candidate and native/full host gate remain operator-owned.
No .yard configuration changed.
