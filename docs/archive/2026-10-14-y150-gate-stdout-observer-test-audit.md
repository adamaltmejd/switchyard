# Y150 gate invocation observer repair

Retain both previous required host failures. The first failed at the image
boundary with seven counted closed invocations against six expected. The
second run's bounded diagnostics show six actual closed-gate passes:
executions 2, 10, 14, 17, 19 and 21. Execution 6 reused a closed result.
Image-adopted implementer 20 ran at plan revision 4.

The wrapper searched for gate markers anywhere in box-exec arguments.
The implementation revision prompt carries the serialized plan and consumed
definition diff. It mentioned the changed closed command and declaration
since the preceding implementer's revision 1. That prompt in Pi's arguments
falsely counted implementer 20 as a closed gate. No seventh closed-gate
execution exists in the reported rows. Production matching is unchanged.
The configuration-load stderr messages remain uncorrelated; no separate bug
is admitted or claimed here.

Each real fixture gate command now emits its literal marker first. This
includes the command that exits 7. A FIFO reader observes actual Pinfold
stdout and appends a count only for a complete line equal to a marker.
Prompt substrings and JSON SDK frames cannot match. The reader forwards
each complete line and any final unterminated fragment to original stdout.
Stderr remains inherited. The wrapper saves the real Pinfold exit status
before waiting for the reader, returns that saved status, and removes its
per-process FIFO on exit. EOF drains the reader before command completion
is reported. There is no held point, sleep, retry or concurrency change.

Test-audit change gate: this repairs the external observer for the existing
G2/G3 matching-reference scenario. Its clauses cover bookkeeping reuse,
selective definition/runtime/image/policy invalidation, unknown-input rerun,
and immutable reference selection beside skip and accepted failure. Its
existing sabotage comments remain: rerun matching gates, match by head
alone, accept unknown input, copy a pass, omit changed consumed identity or
policy, and carry an old operator decision. Counts still come from fixture
commands, independently of Yard metadata. The second host failure exposes
the hardest observer case: an implementation prompted with the real gate
definition diff. No separate assertion or scenario duplicates it.

Every existing count and assertion is retained, including closed 6, other 4
and unknown 5 at the image boundary and the later policy and failure counts.
Zero assertion lines and zero binary lines are removed. The bounded
failure-only diagnostics and previous node-policy repair remain. The prior
metadata and explicit-reference observations were real; policy, skip,
accepted-failure and landing controls still await a successful host run.

The worker runs fmt, clippy and build only. The operator reruns the same
independently runnable required scenario with both prior failures retained.
No host e2e, new ticket, runtime stub, binary test path or runtime witness is
introduced.
