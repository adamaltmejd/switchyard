# Y-54 login-control transition repair

The operator's repair note identifies a second fixture error after the API
approval: `ticket abandon` cannot close a ticket with a live attempt.
`jobs/admit.rs::ticket_close` explicitly refuses it.

The fixture now parks Y-1, abandons its attempt, then closes the ticket
through the existing CLI before reconfiguring for subscription login.
Parking prevents another attempt from starting between abandonment and
ticket closure. The attempt abandonment retires its approval through the
existing lifecycle. The subsequent login ticket runs in the same project.

The earlier commit `adcc8ac` changed the API implementer script to send its
commit before the first completed tool result. That addresses the host
gate's `stopped:unchanged` failure before review. Neither repair changes
the adapter or binding contract.

## Test-audit gate

Keep every G6/G8/G12 assertion, positive control and named sabotage from
the Y-54 scenario. The direct API seat still probes guest credential
absence, publishes without its first stdout proof, then succeeds with a
fresh proven seat. The same-project subscription login still reaches
approval and the fixture still observes its bearer header. Expected values
come from the spec, scripted fixture and pre-call Git/audit snapshots;
observers remain the real CLI, store, host model and guest probe.

No assertion block was added, removed or weakened. This change repairs the
control's setup through supported lifecycle commands, not a new scenario
or a duplicate observation. Zero test lines and zero binary lines were
removed. The operator's requested transition is implemented; no decision
remains pending.

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. Real host e2e
remains the landing gate and was not run in the worker. `.yard/` was not
edited. No runtime pass is claimed for either repair.
