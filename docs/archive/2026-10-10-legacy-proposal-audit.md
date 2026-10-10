# Y-56 legacy proposal and selection audit — 2026-10-10

Review found three remaining defects in 2ce60ab. An ended pre-target planner
keeps open child proposals but no branch identity. Acceptance required that
missing identity and refused the proposal. Explicit default selection also
removed the selected branch's config cache entry, so an in-flight seat on
that branch could lose its publication before the scheduler reloaded it.
A deleted git helper left a misleading doc comment on valid_branch.

Acceptance now uses the current default to classify only an ended proposer
with no target, preserving the old semantics without writing a historical
branch. Known targets still use their branch config; a live missing target
still refuses. Child omission remains default resolution at admission.
Selection no longer evicts the branch cache. The cache already binds branch
and current head, and selection does not change that input. The obsolete
comment is deleted. The spec changes with these fixes.

## Test-audit Gate

The new G15/G8 scenario builds the real landed Y-82 binary from pinned commit
c85369dfaec8f8554da49ddf02a4dbec03bcad02 in a separate build directory. Its
daemon writes the old store through ordinary CLI commands. A real boxed
planner leaves an open child proposal and finishes cleanup. A real boxed
implementer and blocking reviewer leave a stopped:limit candidate. The
fixture verifies planned history, the absent target in the limit payload
and no running executions before stopping the old daemon. It then starts
the current real binary on that same project, accepts the child and overrides
the limit. No fixture code writes or edits the store.

G15's expected parent edge is [3], from the fixture's ticket creation order:
planner, implementation, accepted child. Host store rows show the edge and
the ended target remaining unknown; CLI JSON shows the child's absent
override. G8's allowed override lands on main, proved by host git and CLI
done status, with the blocking override recorded in the store. Its head and
proof arguments are the old pending identity, so the guarantee tested is
that the old exit still accepts that same identity after migration.

The test comment names requiring a target for the legacy ended proposer and
omitting candidate attention target backfill as sabotages. Existing G15
scenarios created planners with known targets. Existing G8 limit scenarios
created candidate identities with targets. They cannot expose this upgrade
failure. This is the hardest legacy case: the worker has ended, its clone
has been cleaned, and the operator still has pending decisions to answer.
It adds no duplicate mount or review-round assertions. Existing G2/G3/G9/G10/
G13 controls and G15's [3,4] assertion stay intact.

The harness can select a real binary only with its daemon stopped. CI now
fetches history so the pinned tree is available without a network fetch in
the test. Both binaries use the temporary fixture's state, socket and runtime
names. Historical compilation uses a separate target directory. No daemon
or CLI has been started here, and no construction state has been opened.

The diff removes 1 harness/test line beside 2 binary lines. This is an
assertion gate, not a suite sweep. No assertion or guarantee was weakened.

## Validation

Fmt and locked all-targets clippy passed. The new migration fixture compiled
but has not run. The suite and runtime sabotage trials remain the host
landing gate, per the ticket instruction. The race fix is verified by source
inspection: selection now performs no cache removal, so it cannot cause the
reported absent-config window. No runtime race result is claimed. No .yard
file, mock, sleep, public model endpoint or test-only path in yard was added.
