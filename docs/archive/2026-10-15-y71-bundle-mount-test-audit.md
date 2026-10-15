# Y-71 bundle directory mount repair

Native validation of c7b855 reached integration execution startup in both
fixtures. Pinfold refused candidate.bundle because its mount host was a regular
file, not a directory. The conflict scenario failed; the partial-work scenario
was still waiting for an unreachable model hold when the operator reported it.
Neither run supplies acceptance evidence.

Candidate staging now creates a dedicated bundle directory for each named
input. Git still validates and exports the exact publication ref and head.
Pinfold receives that directory as the read-only mount host. The guest reads
/yard/candidates/<hex-input>/bundle/candidate.bundle. Source and proof snapshots
keep their separate read-only mounts. Canonical, live writer clones, approval
and source identity handling are unchanged. The binding architecture, worker
prompt and existing fixture fetch path agree. The configuration example has no
bundle path to change.

Test-audit disposition: retain both G2/G9 integration scenarios and all their
original controls. The held-restart setup now observes the integration start
through history, identified by the outside store's attempt/node relation. It
then uses the existing said_before_end seam to wait for the host model's FIFO
signal while rejecting an early execution end or attention. Only after that
signal does it wait for the original latch. The signal is emitted after the
real worker's partial-write tool result, immediately before returning the held
host reply. The latch, store and host partial file still prove the crash point.
No binary barrier, new timeout, sleep, retry or scenario is added.

The changed observation serves G9's held integration and exact bundle import
clause, with G2's named publication bindings preserved. Sabotage: restore the
regular-file mount; startup ends before the signal, so history fails the wait.
The signal bytes come from the fixture host, execution identity comes from the
outside store relation, and expected contents and heads still come from fixture
commands and host Git. This is a repair of the existing scenario's wait, not a
second scenario asserting the same mechanism. No assertion block is removed or
weakened; the old latch check remains. Test lines removed: two fetch-path and comment lines
replaced. Binary lines removed: four bundle path/mount/prompt lines replaced.
No guarantee is removed and no operator approval question is pending.

Validation is limited to boxed cargo fmt --check, cargo clippy --all-targets
--locked -- -D warnings, and cargo build --locked. Host native e2e remains
operator-run; it is not run in this workspace.
