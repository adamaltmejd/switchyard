# Y-94 check observer repair

The operator found an assertion error on candidate 64daabe. The scenario
queried attempt identity on the check table and expected landing child gates
to create checks. Current check identity comes through its execution.
Landing children store judgments on execution and create no check row.
Static compilation did not establish that the SQL or expectations were right.
No focused or full host suite ran on that candidate.

The repair joins check to execution and selects the execution's node and
check's verdict for the named attempt. Literal fixture expectations replace
counts: the full chain has one passing candidate gate and three passing
seats; the reduced chain has a blocking first seat and passing final seat,
then both passing after one repair; the empty selection has no checks.
Landing children remain observed separately by exact node and merged head.
No production or schema change is made.

## Test-audit Gate

1. G3's omitted-gates/no-check clause and G8's skipped-identities/no-fabricated-
   passes clause are observed through the read-only store. G1's full,
   reduced and empty queue controls remain in the same scenario. Existing
   guarantee rows already name these clauses; none is weakened.
2. Sabotage: fabricate a check for an omitted gate or seat, or turn the
   retained first seat's blocking publication into a pass. The literal
   node/verdict sequence differs. The test comment names the sabotage.
3. Full-chain tests cannot detect fabricated evidence for omitted nodes.
   This scenario removes the middle panel seat while accumulating a block,
   then removes every verification node. Its exact landing-child node/head
   assertions observe actual selected execution effects; the repaired check
   assertions independently observe durable publications. Neither replaces
   the other.
4. Expected nodes and verdicts come from the fixture's selected graph and
   scripted model findings, not Yard's resolver or scheduler output. The
   fixture declares one candidate gate and three full-panel seats. The
   reduced panel retains seats 0 and 2, with one P1 block followed by one
   repair and passing publications. The empty selection declares no gate
   or seat. Host git still supplies expected merge parents and tree.

All full-default, retained-block/repair, skipped-node, empty-unreviewed,
protected auto-authority and merged-parent/tree assertions are preserved.
Existing worker refusal assertions are unchanged. This is a focused repair
under the assertion gate, not a whole-suite sweep.

The repair removes three test lines, including the incorrect sabotage-count
comment, and removes no binary lines. No assertion block is deleted.

Validation passed: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings`. Neither focused nor full e2e nor sabotage runs are
performed here. Fresh focused and full operator host validation on the new
head, published review and actual landing gates remain required before
approval. No host validation is claimed.
