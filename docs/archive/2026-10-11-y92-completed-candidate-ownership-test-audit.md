# Y-92 completed-candidate ownership repair, 2026-10-11

## Operator evidence and limits

Exact 5567c874bb5092b2b040003dc1052ead1d4a62a2 host session 33551,
test 248991, ended exit 101: 71 passed, 1 failed, 864.88 seconds.
Both packaged-controller cases, gate-error (including cleanup 2 and
unstarted parent 17), and published-review recovery passed. Only
`a_publication_without_a_registration_does_not_count` failed, before any
review ran. No earlier green transfers to this repair or authorizes landing.

The operator's real CLI history records implementation 1 ending candidate
at 08:07:58.335Z, seq 6; candidate pinned at seq 7; ownership red for that
execution at 08:08:28.339Z, seq 8, after 30.004 seconds; and release at
08:08:39.575Z, seq 9. No review or approval arrived before the existing
600-second watch failed. The full terminal block remains on the operator
host at /tmp/switchyard-y92-5567-full-host.log. It was not available here.
The fixture was removed before the read-only lock/store snapshot. Whether
box listing or host lock caused the first unknown probe remains unknown.
A separate live kernel observation establishes shared Podman image-lock
contention between a durable-package owner and gate-error listing. It does
not establish the unobserved noreg probe's cause or the historical
1751/Y-95 cause. This repair makes no runtime-delay claim.

## Source diagnosis and repair

Checked release already proves box absence and inherited host lock freedom,
then sets teardown and clears reservation in one daemon transaction. Reap
does not end an already-ended candidate again. Before this change only
completed cleanup could reconcile its own ownership question in release;
the ended candidate's question blocked readiness after reservation cleared.
Readiness correctly refuses open questions and remains unchanged.

The same transaction now rereads the released execution through the existing
authoritative result eligibility predicate. Only a supported implementation
with candidate outcome, an eligible result and an existing current inherited
lock can reconcile its exact red/ownership/execution item. Existing checked
box absence and host-lock freedom remain prerequisites. No result, candidate,
input, judgment, publication, lifecycle, owner or eligibility predicate is
created or copied. Running controllers, legacy contracts, non-candidate
outcomes and unrelated/error/stopped questions retain existing behavior.
Completed-cleanup and interrupted published-review repairs remain intact.
Binding G5/G8 and current release prose are updated in this commit.
No dependency is added. Existing package and OS dependency reasons and pins
remain unchanged; packaged availability still does not select the daemon
controller before Y-93.

## Test-audit Gate

Disposition: extend the existing unregistered-publication hardest case;
retain every original assertion and both original review executions. No
new scenario, assertion deletion or suite-size change.

1. G5 shown-by now names completed-candidate ownership reconciliation after
   a real executable-availability failure and successful box absence with
   the inherited lock held. G8 shown-by names automatic continuation of the
   same candidate into the original unregistered then registered reviews.
   The initial FIFO block identifies the real implementation box and proves
   its existing lock is held. The refusal block requires candidate outcome,
   reservation, ineligibility, the exact open ownership item and no review.
   The allowed block requires exact reconciliation, eligible unchanged
   result/bindings and one implementation, followed by the original two
   review checks and sole second-check approval.
2. Named sabotage in the test comment: omit completed-candidate ownership
   resolution and approval cannot progress; resolve on box absence alone
   and the held-lock reservation/no-review block fails. The existing
   registration sabotage still approves the wrong first check. No timeout
   counts as proof or pass. Sabotage execution requires the operator host;
   no e2e or sabotage run occurred inside the candidate.
3. Existing cleanup recovery settles an ended removal without a candidate
   result; interrupted published-review recovery settles a registered
   publication after restart. Neither exposes an already-successful
   candidate stuck before its first review. This extension carries that
   exact boundary in the existing hardest registration scenario, without
   a twin, a crash, a prompt selector or an overloaded-runtime timing bet.
4. Expected identities come from the fixture's one implementation and its
   externally observed real down box name, associated by read-only SQLite.
   Candidate outcome and ownership/reconciled tokens come from G5/G8.
   The fixture opens the actual existing inherited lock and proves conflict
   before controller completion; it reacquires that same open inode after
   the unknown listing failure. Host metadata compares the open descriptor's
   device/inode with the still-current path before acquisition. This expected
   identity comes from the actual inherited host file, not Yard's predicate.
   It creates no replacement private path.
   The fixture-owned link runs installed pinfold, preserving real arguments,
   stdout and exit status. Removing it gives an actual exec failure, never
   fabricated list output. Two FIFO-held successful real listing replies
   establish that a prior checked probe completed while the host lock was
   held. Restoring the link precedes that control; unlocking while the next
   real reply is held permits genuine checked proof. Supported CLI attempt
   show supplies externally observed result/bindings before and after;
   equality is intentional identity preservation, not a copied predicate.
   SQL observes no reviews and one original implementation. Approval's
   two-check/second-only expectation remains the original fixture oracle.

The fixture writes only ordinary host tool links/FIFOs and uses OS locks,
real Pi/model traffic and actual pinfold commands. It writes no private
attempt input/store, substitutes no harness/model frames, adds no binary
hook, sleeps, retries, deadlines or thread limits. No runtime process is
pruned, restarted or killed. The deterministic holds last until explicit
fixture release. Every original gate-error and publication-recovery byte
outside this function remains unchanged.

## Validation and remaining gates

Candidate checks passed: cargo fmt --check, cargo clippy --all-targets
--locked -- -D warnings, cargo build --locked and git diff --check.
All 13 guarantee rows outside G5/G8 compare byte-for-byte with HEAD.
Only the target test function and its sabotage comment changed; gate-error
and published-review recovery functions remain byte-preserved. Binary
lines removed: 1, replaced to extend the existing exact resolution branch.
Test lines removed: 8 wrapper construction lines, replaced to retain its
registration filter and add real tool holds; no assertion block removed.
No boxed e2e,
image build, host install, push, publication or operational daemon change
is performed. All 15 guarantee rows, package/CI/pins/native bridge and
Y-97/Y-98 controls are retained; old archives and repository .yard untouched.
Prior review 923's generic missing-path P3 diagnostic remains uncorrected.

Fresh exact-head source/test audit, focused changed registration case,
existing gate-error and published-review controls, package pair, full 72,
published review and successful actual landing remain required. External
runtime delays, if still blocking those gates, must be reported concretely;
this source fix is no host acceptance claim. Y-93, parent Y-64 and release
through Y-80 remain required; Y-81 remains parked. No new proposal is made.
