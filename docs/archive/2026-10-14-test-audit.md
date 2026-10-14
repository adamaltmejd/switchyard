# Y-147 diagnostic preparation

Y-147 is not fixed. No actual lock holder or errno was available in the
retained failure report. The referenced operator evidence directory is
absent from this worker. Host e2e and CI are operator-run after approval.
No inheritance hypothesis was treated as established.

The daemon still takes the same exclusive, nonblocking flock and holds it
for its lifetime. Its failure now reports the actual numeric errno and
the device/inode of the opened file. It reads no process arguments or
environment for this diagnostic. Execution and host locks are unchanged.

The Linux fixture reports host observations before kill, after a successful
wait reaps the owned daemon, and before signalling startup EOF to the test.
It reads only lock metadata, matching `/proc/locks` rows, process parent
PIDs and start ticks, and matching descriptor flags and lock rows. The
snapshot includes the fixture-owned PID and matching lock-owner identity.
It records process identities on both sides of descriptor inspection and
checks that the descriptor still matches the device/inode. Observations
can race; they are not a new ownership authority.

Each proc file read is capped at 256 KiB. A snapshot inspects at most
32,768 processes and 131,072 descriptors, retaining at most 128 matches.
It reports capped scans, inaccessible processes and disappearing descriptors.
Missing evidence is not proof of absence. The observer emits to test stderr,
which Cargo retains for failures. It adds no dependency, sleep, retry,
timeout, binary barrier or runtime flag. Other platforms omit proc snapshots.

## Test-audit Gate

Disposition: keep every assertion block. G2 frozen plan evidence, G7 frozen
usage history and all G5 crash/restart assertions remain unchanged. There
is no new guarantee or Shown by clause in this diagnostic preparation.
Expected values, held crash points and sabotage comments are unchanged.
The fixture adds observations only; none become an assertion or an oracle.
Binary lines removed: 2. Fixture lines removed: 2. Test assertion lines
removed: 0.

The Gate's four questions for a new assertion block do not apply: no block
is added or changed. A completed fix must pass that Gate, including the
same-project live-daemon refusal and killed-daemon restart controls named
by Y-147. This candidate claims neither new control nor a completed fix.

## Remaining operator work

Retain CI38491717533 and its failed logs. The original source was
0fdbe7179dc24db2e899ba1c8e87c89cc6ca51c5 with Pinfold 0.2.3:
x86_64 77/79 in 614.76 s; arm64 78/79 in 431.91 s.

Run the existing host suite and exact Linux x86_64/arm64 CI with this
diagnostic preparation. For each refusal, retain the daemon's errno/dev/inode
and the same fixture's before-kill, confirmed reap, after-reap and startup-EOF
records. Match the daemon's opened inode with the kernel rows and matching
descriptors; compare owner PID/start ticks with the reaped fixture PID/start
ticks. Account for every missing, raced or capped observation before choosing
a fix. Do not infer inheritance solely from a refusal or an open descriptor.

Only after that evidence identifies the mechanism can Y-147 choose the
smallest fix and verify frozen history, ownership and live-writer refusal.
The installed construction daemon and `.yard/` were not touched.

## Worker validation

Passed `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, and
`cargo build --locked`. The e2e suite was not run, as instructed by Y-147.
No lock observation from a real failing host run is claimed by these checks.
