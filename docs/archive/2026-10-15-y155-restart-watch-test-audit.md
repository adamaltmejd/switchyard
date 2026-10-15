# Y-155 restart history observer repair

The operator's complete focused native diagnostic at exact candidate
9466a7f reached revision-two selection and approval 4 at audit sequence 132.
The old history client was dead after the committed-selection restart.
The test waited on that old channel and never issued the candidate edit.
The operator deliberately interrupted the diagnostic after proving this
invalid observer. It was neither a pass nor an asserted test failure.

Drop that history Watch before killing the daemon. After startup, open a
new project history Watch from recorded sequence zero. Revision-two approval
already has an exact revision predicate, so replaying earlier history cannot
satisfy it. Keep every assertion and the same concurrent execution schedule.

Audit every restart transition in the affected join scenario:

- The first held-right restart used no Watch until opening a new client at
  the recorded publication sequence. Move its old Watch drop before the
  kill; keep the recorded sequence and new client unchanged.
- The held-owner restart already drops its Watch before kill and opens a
  new history client from zero after startup. Keep it unchanged.
- The committed-selection restart now drops and reopens as described above.
- The later five-writer helper receives no Watch from the join scenario.
  Its restart uses the old client only to receive the external owner's FIFO
  message while the daemon is down, then drops it and opens a fresh history
  client after startup. It reads no post-restart history from the old client.

Test-audit Gate disposition: retain all affected G2/G5/G8/G13 blocks.

- Exact committed publication settlement, ownership and historical inputs
  prevent replay and stale evidence from authorizing later effects. All
  count, full-row, source identity, refusal and landing assertions remain.
- Existing sabotage comments cover replay, stale selection, erased raw
  findings and early ownership release. The watcher repair restores the
  external observer needed to reach those assertions; it adds no assertion.
- Use the existing real hard case. No added scenario, machine, build, box,
  model turn, retry, sleep, deadline change, barrier or product change.
- Expected events come from the real recorded project audit history through
  status --history --since. Fixture revisions and public-store rows supply
  exact expected identities, unchanged by this repair.

One test line moved; two added. No assertion or binary line removed. No
product control or Shown by change. Worker fmt, locked all-target clippy
with warnings denied and locked build passed. No worker native
run. The operator will run this entire focused case at the repaired exact
candidate before approving another full prepared host gate.

The operator's evidence remains at
/home/yard/validation/y155-9466a7-join-20261015-evidence: native.log,
terminal.json, confirmed-stalled-store.sqlite, watch-child-observation.json
and operator-interruption.json. The operator reports that interruption
followed proof of no running/reserved fixture executions and matched the
disposable daemon by executable, parent/start identity and actual daemon-lock
inode. Construction services and unrelated boxes were untouched. This record
preserves the supplied diagnosis without claiming independent verification
of host evidence from the worker. Earlier host failure records stay intact.
