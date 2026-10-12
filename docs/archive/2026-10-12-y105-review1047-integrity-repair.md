# Y-105 review1047 ownership integrity repair

Source inspection found two predecessor ownership checks. The answer path
refused open mutation intents; prepared continuation did not check them in
its own proof. The top-level queue also holds open intents. This is a source
integrity defect, not evidence of a reproduced bypass.

Both paths now call one proof over the existing landing resumed provenance
and its children. It validates provenance and requires ended status, absent
reservation, the current lock contract, absent live boxes, free inherited
host locks and no open mutation intents. Failure to inventory boxes remains
unknown ownership and refuses progress. The proof returns the existing child
rows for continuation. No owner, receipt, lifecycle or decision store was
added. The duplicated answer check was removed.

The late answer transaction now checks ended attempt liveness separately from
approval. Its stale attempt identity compares the named attempt ID with the
current live attempt ID or null. Approval is named only when its state changed
from active. Ending an attempt no longer reports active versus active approval.

All existing full and reduced selection, acceptance, restart, infrastructure
retry, refusal, remaining-gate and failure-preservation controls are retained,
including b67d56c's real landing-attention observer. No test or assertion was
changed. No mock, sleep, retry masking or binary barrier was added.

## Retained evidence and validation limits

The original landing1965/host1969 failure remains: 72 passed, three failed in
922.03 seconds. The Pi-extension failure explicitly reported SQLite database
or disk full. The Codex config/link failure lacked a native terminal frame;
the gate-error observer received empty CLI JSON. The source observer defect
was repaired in b67d56c, but the supplied host result does not establish the
exact runtime cause of either latter failure or a shared cause.

The operator reports retaining four inactive historical validation directories
under /home/yard/validation/retained-tmp-20261012 with original-path symlinks,
after process-reference and content/mode/ownership checks. The host's separate
7.7 GiB /tmp tmpfs then had 6.2 GiB free. No repository capacity workaround was
introduced. An independent native investigation was observed during that gate;
no causal load claim is established.

Review-timeout evidence also remains unresolved. This repair makes no claim
that the timeout was caused by the duplicate proof or that it has been fixed;
no detailed timeout result was supplied in this repair request.

Worker checks passed: cargo fmt --check and cargo clippy --all-targets --locked
-- -D warnings. The worker ran no host e2e or sabotage. Fresh actual merged-host
validation and the original sabotage controls remain operator-owned. The full
original Y-105, Y-104, Y-96 and Y-61 scope remains required; this record does
not close those contracts.
