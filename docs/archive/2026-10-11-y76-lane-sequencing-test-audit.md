# Y-76 fixture lane sequencing audit

The operator's source gate identifies an unsatisfied fixture precondition in
`a8fe459`: approval retains a live lane, while the fixture has two lanes.
Creating three unretired tickets cannot admit the third control. This is a
source diagnosis, not an observed host failure of that candidate.

## Repair and identities

After all observations for each fixture ticket, use supported CLI operations
in order: park the ticket, abandon its attempt, abandon the ticket. Parking
prevents readmission. Attempt abandonment resolves attention and releases its
lane in the daemon transaction. Ticket retirement follows that transaction.
Wait through `status --history --since` for cleanup's execution-ended event
before admitting another ticket. Cleanup removes transcripts, so all raw
frame and execution comparisons precede retirement.

The original attempt and executions 1 through 4 are unchanged. Retiring Y-1
creates cleanup execution 5. Y-2's fresh nonzero execution is 6 in attempt 2;
its cleanup is 7. Y-3's fresh zero execution is 8 in attempt 3. The wrapper's
explicit-zero transformation now targets execution 8. Waiting for cleanup
makes this numbering independent of scheduler ordering. Retire Y-3 only
after its zero observation, too. No capacity change or construction config
edit is needed. G7's shown-by row describes the sequential controls.

## Test-audit Gate and dispositions

Keep every accounting assertion block. Remove no assertion or guarantee.
The only changed assertion arguments identify the fresh executions after
cleanup; their meaning and external expected values are unchanged.

1. G7's optional measured usage clause requires fresh nonzero and numeric
   zero controls alongside unknown resumed local estimates. The same single
   scenario retains the missing-cost, failed-resume, interruption, reopen
   and resumed-success chain.
2. Its existing comment names the sabotages: storing cumulative cost as
   local, inventing a delta across missing/interrupted evidence, discarding
   fresh estimates, defaulting absent cost to zero, and adding terminal
   totals or resumed history. Both fresh assertions still catch these with
   numeric observations and complete harness cost metadata.
3. These controls distinguish unknown local cost, observed numeric zero and
   a positive fresh estimate. No repeated full scenario or authentication
   matrix is added. G8's independent authentication controls remain intact.
   Cleanup waiting is fixture sequencing, not another cleanup assertion;
   G7's existing landing scenario already tests cleanup's guarantee.
4. Expected USD remains the actual transformed fresh harness result, compared
   numerically. All original cache/token expectations remain host fixture
   counters; interrupted output remains the maximum actual raw report for
   the single provider response. Provenance/completeness comes from G7.
   Resume identity and persistence/reopen assertions are unchanged.

Retained blocks: initial missing cost; failed resumed terminal tokens and
unknown cost; interrupted raw output, cache tokens and partial metadata;
reopen persistence; resumed terminal replacement and unknown cost; raw
positive/increasing session totals; independent fresh nonzero and zero
cost/provenance controls. None is weakened or duplicated. Source delta:
three test lines replaced, fifteen added; zero binary lines changed.

## Validation

`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `cargo build --locked` pass in the worker box. No e2e or sabotage run,
public model request, development daemon or construction configuration edit
occurred. This is a source repair only. Exact-head focused/full host
validation, fresh review and landing remain operator-owned and pending.
