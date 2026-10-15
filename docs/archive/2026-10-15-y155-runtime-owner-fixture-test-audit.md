# Y-155 runtime-owner fixture correction

The operator's real host preflight of f6592ab exposed an unreachable hold in
the existing multi-review join scenario. The wrapper matched `box down`.
Successful review teardown instead calls `LiveBox::down`: close the retained
`box up` stdin, drain its real terminal frame and wait for that process.
It never invokes `Pinfold::down`. Reported store evidence showed selector16
ended selected with reserved=0, teardown=1 and no ownership attention while
the test waited for picker-owner-said. This is a fixture failure, not proof
of the ownership repair. The previous dated Gate describes the intended
mechanism; this correction records the actual call path.

The operator retains the running preflight at
`/home/yard/validation/y155-f6592a-join-20261015-evidence`, with store at
`/tmp/ye-4158112-g4-graph-join/p/.yard/local/store.sqlite`. Neither path is
mounted in this worker. The reported state and evidence locations are also
recorded in candidate proof and progress. No terminal failure output was
available to this worker when this correction was written. The host evidence
is left untouched, including its eventual terminal failure.

Gate disposition: replace the broken hold inside the existing scenario.
Keep its assertion blocks and all existing controls. No product code changes.

- G5/G8 Shown by: a successfully completed registered finding selection
  retains its exact ownership item and reservation while a real runtime
  descendant holds its inherited effect lock across restart after box removal.
  Release reconciles only that item without replay or publication mutation.
  The living G5 clause now calls this a runtime descendant.
- Sabotage: clearing ownership while the inherited lock remains held, or
  recognizing only pass/fail at checked release. The existing comment names
  both. Matching the unused box-down path instead also fails immediately when
  the selected execution ends before the fixture reports its holder.
- Existing pass/fail and interrupted-publication scenarios cannot detect
  omission of successful typed selection completion. This holds the exact
  registered selected execution across restart, rather than its first turn.
- Expected values come from the fixture box identity, real runtime exit,
  child PID and descriptor inode comparison, host lock attempts, box inventory,
  original store report/result and exact attention identity. The same-project
  release still reaches approval and the original raw-source, revision,
  single-selection, candidate-change and landing controls remain.

`Pinfold::command` inherits the current effect FD through `git::inherit`,
which clears close-on-exec only on that actual descriptor. The wrapper now
delegates real `box exec` unchanged and retains the descriptor in a host child
after the real command exits successfully. The child compares `/dev/fd/*`
to the exact effect lock with the host's same-file predicate, reports the
matching descriptor and blocks on its release FIFO. Only its stdio is
redirected. The parent wrapper waits for a host acknowledgment before exit.
The harness waits for that exit before normal live-up stdin-close teardown;
real runtime frames and exit codes remain intact.

The observer begins at the audit sequence of the fixture-held publication
before releasing it. `said_before_end` observes the holder before acknowledging
wrapper completion; an already ended picker fails immediately. Restart occurs
only after successful result, registration, box absence, live child, held lock,
reservation and exact ownership attention are proved. After restart the same
child, reservation, result and ownership item remain, with no approval. FIFO
release permits the existing checked reconciliation and same-inode lock freedom.
There is no sleep, retry, timeout-as-pass, runtime stub, binary hook or lifecycle.

Test diff: 42 added, 15 removed lines. Binary diff: zero lines.
Removed lines are the unused box-down hold and blind FIFO observer, not
assertion coverage. No scenario or guarantee is removed.

Worker gates: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings, cargo build --locked. Native execution remains the operator's
sequential real host gate; it is not run inside this worker.
