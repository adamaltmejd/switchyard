# Y-58 exact-candidate audit repair

The operator audited cfd73a1f12cdb8b4d4e9a8a8b108d2850445f6b9.
Its disconnect observation killed the adapter through `Operator::drop`.
That proved process death, not stdin EOF. The original dated evidence
claimed too much. This file corrects that claim; the old archive is kept.

Two further findings came from source inspection, not CLI reproductions:
attention bounded only its notification wait, and stdio errors dropped
accepted forwarding tasks. The repair preserves daemon ownership,
execution lifecycle, worker grants and exact identity decisions.

## Source dispositions

- Attention now puts the entire poll inside one 25-second timeout. Both
  `jobs::load` and `ready_configs`, their Git awaits, the store observations
  and notification waiting are inside that future. Expiry reads the current
  audit seq and returns no attention with `idle: false`. Incomplete config
  or readiness reads cannot establish idle. Git's ordinary read children
  retain `kill_on_drop`; the timeout drops the pending read future. This
  is source evidence of the bound, not a measured deadline claim. No test
  waits out the poll deadline, and no binary barrier or test clock was added.
- The stdio task set is owned outside the I/O loop. EOF, read errors,
  oversized frames and write errors all leave through that loop into the
  same wait-release and task-drain path. Accepted forwards are awaited;
  writable stdout receives their results. Failed output suppresses further
  responses without stopping the drain. An oversized frame gives a normal
  JSON-RPC invalid-request error before input is stopped when output works.
- MCP shuts its runtime down in the background only after accepted forwards
  have drained. An outstanding blocking stdin read after a stdout failure
  cannot hold adapter exit. No execution stop is sent on these paths.

## Assertion gate

G7's sole disconnect proof is replaced, not weakened. The same real boxed
Pi worker stays held by the scripted host model. The second trusted client
closes its actual stdin, waits for stdout EOF and observes successful process
exit before the model hold is released. The worker then reaches approval and
landing. Expected exit success comes from the stdio contract, and the head
and file contents come from Git and the fixture. Sabotage: stop work on stdin
EOF, or leave read waits active when stdin closes. Killing the adapter does
not exercise either path. A deadline in the observer is failure, never proof.
The initialized-session worker refusal, stale identity and schema blocks
are unchanged.

One new G7 scenario holds a real sync at its Git update-ref before effect.
The host sees the canonical ref unchanged, sends an oversized frame through
the real client pipe and receives JSON-RPC -32600 with null id while the
child is still held. After releasing Git, the host receives the accepted
sync's successful completion, observes canonical at the independently read
checkout head and observes adapter exit 1. A prior successful sync in the
same project is the positive control. The scenario uses real Pi work and
queue landing before sync, with the scripted host model and real boxes.
Sabotage: drop or abort accepted forwarding tasks on transport failure; the
accepted sync loses its completion response. The earlier wait scenario has
no in-flight mutation and cannot show this. Input and output errors share
the same drain owner, so separate repetitions of that assertion are not
added. No assertion names error prose, argv or a binary file layout.

Diff line accounting: 8 test lines removed beside 54 binary lines removed;
these are replaced helper, EOF and poll lines. No guarantee is removed.

## Validation

`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `cargo build -p yard --locked` passed.
An isolated real-pipe smoke check also observed adapter exit 1 after stdout
closed while the client's stdin remained open. Its observer deadline was
only a failure bound. It used a fresh XDG state path and no daemon or project.
Host e2e and sabotage runs were
not executed in the worker, as instructed. The separately running host
suite at the old exact head cannot establish this repair. Pinfold is absent
here. No .yard configuration or construction backlog was opened or changed.
