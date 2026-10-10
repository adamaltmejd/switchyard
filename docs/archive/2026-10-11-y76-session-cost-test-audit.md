# Y-76 pinned session cost repair

## Actual host evidence

The operator reports exact `ab95eef` focused host validation failed only
the raw estimate proportionality assertion: exit 101, zero passed, one
failed, 59 filtered, 21.46 seconds. Numeric zero and every earlier phase
passed. A separate test-side diagnostic printed real unmodified result
frames and failed at the same assertion after 17.65 seconds. That diagnostic
patch was restored and clean exact `ab95eef` verified. Neither was a pass.

The reported pinned Claude frames distinguish invocation tokens from cost:

| Execution | New input/output/read/write | Session USD | modelUsage input/output/read/write |
| --- | --- | --- | --- |
| 1 | 2000/200/140/60 | 0.009267 | 2000/200/140/60 |
| 2, failed resume | 1000/100/70/30 | 0.0139005 | 3000/300/210/90 |
| 4, success resume | 2000/200/140/60 | 0.0231675 | 5000/500/350/150 |

`modelUsage.costUSD` agrees with `total_cost_usd`. The amounts are session
totals, not new execution cost. Earlier source and archive statements that
result cost was invocation-local were false. They remain historical records;
the binding contract is corrected in this candidate. The operator's prior
captured interruption establishes two same-id assistant snapshots reporting
output zero, with no result or later nonzero report.

## Implemented limit

The supervisor reads resume identity from the existing execution row before
launch and passes that fact to its reader. Claude retains invocation-local
result tokens and partial assistant observations, but resumed result cost is
null with unknown/unreported metadata. A session total is not a report of
local cost. No prior missing value is treated as a zero baseline, and no
interrupted execution or previous row supplies a guessed difference. Fresh
executions retain finite nonnegative reported estimates, including zero.
They retain existing success/failure completeness rules.

No invocation-start cost baseline is observed or reconstructed. This reader
does not recover resumed local cost from session files or modelUsage. That
limit preserves optional data and increments-only recording. There is no
new store, schema, lifecycle, ownership, grant, dependency or billing model.
Pi and Codex accept the reader's additional argument without using it;
their normalization behavior is unchanged. The daemon remains the writer.

## Test-audit dispositions and Gate answers

Keep the single hardest common scenario's first four phases. G7's row now
names missing-baseline resumed costs and independently necessary fresh cost
controls. Each block has an external observer and expected value:

- Initial missing cost still requires null/unknown, despite a positive raw
  harness estimate. The wrapper's omission is the expected availability.
- Failed resumed result and final resumed success now require null/unknown
  cost while retaining every token/cache, resume identity and completeness
  comparison. Saving the cumulative result as local, or inventing a delta
  across missing/interrupted evidence, fails these execution JSON blocks.
- Interrupted output still comes from every actual raw report for the one
  provider response. Multiple content blocks, EOF without result, partial
  metadata and exact persistence after reopening remain unchanged.
- Raw positive cost and increasing resumed totals establish the observed
  cumulative trigger. Their ordering is external frame evidence. The failed
  result has fewer new fixture tokens than the fresh result but greater raw
  cost. No raw session amount becomes an expected execution amount.
- A new fresh ticket retains the real nonzero reported amount and complete
  harness cost metadata. Expected USD comes directly from its real result;
  this controls against an implementation that discards all Claude costs.
- Another fresh ticket receives the explicit-zero measurement transformation.
  Its numeric cost must equal the real transformed result and be complete
  harness data. Both expected and observed amounts require numeric conversion;
  null cannot pass as zero. This controls the initial missing-cost case.

The invalid raw proportionality oracle is removed. It assumed local cost
and failed precisely because the pin reports session totals. Its replacement
is acceptance of truthful null execution cost alongside demonstrably nonzero
raw cumulative evidence, not acceptance of a larger execution amount. The
numeric fresh controls reuse only cost comparisons; they do not repeat the
four-phase accounting scenario or an authentication matrix. The original
missing baseline and interruption are preserved, not replaced by a fixture
with known prior prices. All expected tokens/cache counts still come from
host fixture counters. Cost metadata comes from G7. No price oracle remains.

The test comment names storing cumulative cost, subtracting without coverage,
and dropping fresh reports as sabotages. Existing Pi/Codex and G8 route
controls cannot catch this pinned cumulative-cost mechanism. Fresh nonzero
and zero controls observe distinct necessary classes: a measured amount,
an observed zero and missing local evidence. No twin scenario is added.
The operator's original-scope defect is addressed without a guarantee waiver.
Test source lines removed: 61; added: 66. Binary lines removed: 11; added: 27.
These counts include reader signatures and collapsed obsolete call arguments.

## Validation

`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `cargo build --locked` pass. No host e2e, sabotage run, development
daemon, public model call or construction configuration edit occurred here.
The new fresh controls and repaired resumed-cost records still require
operator exact-head focused/full host validation and fresh review/landing.
No passing host result for this repaired candidate is claimed.
