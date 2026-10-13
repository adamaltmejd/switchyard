# Y-128 stopped-owner fixture synchronization

The operator reported native qualification of candidate
c5153f4d3e9b8b14376ce081ed10a75e5df3a225:
- Focused unchanged control passed 1/0 in 121.68s.
- Removing only `|| stopped_implementation` failed directly at
  g08_durable_package.rs965 in 115.40s: the old ownership item was open/null,
  rather than resolved/reconciled. It failed before the successor FIFO.
- Exact production restoration left capacity.rs SHA256
  77b6e060aa8d23072e27da7a4ffe3dc5ef62fec1fe9f7bac89ab548f722ea02b
  and an empty tracked production diff. The restored control failed in
  134.98s at g08_durable_package.rs921 with old_lock.try_lock WouldBlock.

The restored-control result remains failed, alongside all four original
failures. Operator evidence lives at
/home/yard/validation/y128-c5153f-host-20261013/operator-{control,mutant,restored-control}.log,
operator-stopped-release-mutation.patch and operator-report.md.
These paths are operator-reported; they are not mounted in the worker box.
No historical fixture or evidence was recreated or mutated here.

## Ordering and correction

run_harness stops reading and returns on the supported stop signal. Its real
pinfold exec_streaming child is not killed on drop; taking the box down cancels
the guest. The host wrapper and its stream observers may still be draining
when implementation ends and capacity::release issues its real listing.
A successful listing reply orders neither those host exits nor inherited-lock
freedom. The previous test incorrectly treated it as that boundary.

The same real listing FIFO now holds the release probe while the fixture
observes actual box/process absence and verifies the old descriptor's device
and inode. A fixture thread calls native blocking File::lock on that descriptor.
It acknowledges successful acquisition through lock-said-3 only after the
kernel grants the lock. The returned File retains it across the thread join.
Metadata is checked again after acquisition. This orders the fixture's new
owner after the actual inherited owners leave, with no polling or retry.
The blocked probe cannot mistake the handover for freedom: the fixture owns
the lock before releasing the listing reply. Its first probe must refuse.
The next listing remains held until that owner unlocks and the existing
native free-lock assertion succeeds. Checked release and the direct exact old
attention assertion still precede said-4. A lock that never becomes available
fails at the existing FIFO observation deadline; timeout supplies no pass.

This changes only the initial stopped-controller segment. The later selected
publication, separate published-result ownership hold, all later assertions
and the entire G4 startup journey are unchanged. The packaged scope from
 e845087 uses the recorded implementer node and authoritative admitted runtime;
ordinary preparation retains its explicit ownership exit.
No binary, binding promise, API, lifecycle, configuration or deadline changes.
The historical stale-probe chronology remains unknown.

## Test-audit Gate

Keep the existing G4 reservation/subsequent-work, G5 stopped packaged owner,
and G8 stopped-controller/subsequent-publication clauses. Kernel acquisition
is the host-owner oracle; the FIFO acknowledges that actual acquisition,
not a copied Yard decision. Native pinfold and the box process probe remain
independent guest/box observations. File metadata supplies same-inode proof.
The old exact execution identity, public store and fixture publication remain
the resolution and same-project control oracles.

Restoring completed-only reconciliation still sabotages the direct obsolete
attention assertion. No existing block observes that item at checked release
before the successor wait. Holding both teardown and inventory while the real
streaming host child drains is the difficult ordering, rather than an
unopposed first release. The correction retains the free-lock requirement and
orders it with the kernel instead of assuming it from an unrelated event.
It replaces 2 test source lines with 14; no assertion block or binary line is
removed. No new scenario, sleep, retry or product barrier is added.

Operator qualification remains required after review: focused repaired G8,
unchanged G4 stopped startup, direct mutation failure, exact restoration and
unchanged control, then the unchanged full locked host gate with no other host
gate running. The worker must not run host e2e in a box. Neither the Pinfold
init timeout nor earlier Pi/Codex failures is resolved by this correction.

Worker verification passed: cargo fmt --check,
cargo clippy --all-targets --locked -- -D warnings,
and cargo build --all-targets --locked. No e2e scenario ran in this box.
