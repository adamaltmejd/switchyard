# Y-58 repaired-head audit

The operator audited 280c5bf. Its complete attention bound and mutation
drain were accepted, but its sync control read the wrong response layer.
Its expiry cursor could also hide attention raised during incomplete
readiness reads. These are repaired here. Earlier archives remain dated
evidence, not the current contract.

## Cursor disposition

On attention expiry, `seq` now retains the caller's `since`, or zero if
absent. No store read advances it after the incomplete poll. Attention is
empty and idle remains false. A successful poll still returns its observed
audit sequence with its items or established idle result.

Source evidence: freshness checks an open item's `attention.raised` audit
sequence against `since`. Retaining that same cursor leaves an event raised
after it eligible on the next poll. Zero also keeps pre-existing items
eligible when the initial request omitted `since`. Reading last_seq on
expiry could instead move past a raised item that was never returned.
The timeout still encloses config and readiness Git awaits. No timer test,
clock override, store edit or binary barrier was added. This is source
evidence of cursor integrity, not a runtime reproduction or measured bound.

## Assertion gate

G7's held-sync transport-failure clause keeps its real Git hold, host Git
before and expected heads, refusal before release and completion after
release. The allowed sync now uses `send` and `receive` to observe the raw
MCP `result.isError`, rather than the daemon payload extracted by both
helper layers. Expected false comes from the successful MCP tool contract.
This repairs the positive control in the existing scenario; it adds no
twin scenario. The test's existing sabotage aborts accepted mutation
forwards on transport failure and loses the completion after release.

The final exit assertion observes a normal non-success exit, as the
operator requested, rather than the arbitrary numeric value 1. A signal
exit or successful exit fails. Its expected value comes from the binding
transport-error contract. The EOF and worker refusal, stale identity and
schema blocks are unchanged. The held mutation remains the hardest case
for the drain mechanism; the wait-only scenario has no accepted mutation.

Diff line accounting: 3 test lines removed beside 3 binary lines removed.
No guarantee or shown-by clause is removed.

## Validation

`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`,
`cargo build -p yard --locked` and `git diff --check` passed. Host e2e and
sabotage runs remain
deferred to the landing gate at the operator's instruction. No .yard
configuration or construction backlog was opened or changed.
