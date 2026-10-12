# Y-106 restart history observer repair

The operator inspected the unchanged focused G2 scenario on exact candidate
`4b953a0e38a70b6fdad93677b50d8bf812298ea2` during its host run. It reached
the accepted first omission, held host child, external target advance,
daemon death and fresh preparation. The original omission on parent 17 had
attention 4 resolved accept. Fresh preparation 21 bound the advanced
old head; attention 5 was open and parent 21 was ended fail, reserved 0.
These IDs describe that observation only. They are not fixture constants.

The observer had no live history reader. A zombie Yard child remained until
Watch drop, while fresh audit data already existed. Project history uses a
child `status --history` process whose CLI returns when the daemon API call
fails. Keeping its Watch across daemon restart cannot read new events.
This is an observer defect, not evidence of a production stall.

This record describes the live observation. No terminal result or elapsed
time for that old focused run was supplied. Its exact-head evidence remains
independent of any later run; the earlier failed input-declaration control
and its audit record also remain unchanged.

The repair covers all four added restart boundaries: the held host child,
accepted landing cleanup, explicit-empty pending question and empty landing
cleanup. Before each restart it retains the consumed event cache and last
consumed sequence, then drops and reaps the old Watch. After startup it
creates a new project history observer from that sequence and restores the
cache. Unread durable events are replayed rather than skipped by taking the
store's latest sequence. Cached pre-crash identities and snapshots remain
available to the existing exact attempt, preparation and successor filters.

Scoped test-audit disposition: repair the existing G2 observer. No assertion
block is replaced or weakened. G2's stale old identity/no-write refusal,
immutable decision and fresh acceptance, G3's checked release and exact
empty pre-intent decision, G8's absent omitted judgments, and G1's ordinary
controls remain required. The operator observation does not establish the
later assertions: delayed refusal, fresh acceptance and actual landing,
ordinary reduced/empty controls, or explicit-empty restart and landing.

Expected heads still come from host Git, node names from fixture literals,
ownership from the external lock, and persistence from unchanged snapshots.
The real host child remains held at the crash, with its external Git and
store state proved before daemon death. Existing sabotage comments and
positive controls remain. Y-105 failure and infrastructure-error controls
and production ownership, identity and CAS guards are untouched.

Added 21 test lines; removed zero test or binary lines. The shared Watch
helper, deadlines, concurrency and runtime are unchanged. No reconnect
retry, sleep or binary barrier was added. Worker checks are fmt and clippy;
no host e2e was run here. The operator must rerun the exact repaired control
and meaningful sabotage. Y-106 and Y-104/Y-96 integration remain required.
