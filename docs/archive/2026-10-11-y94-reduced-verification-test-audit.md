# Y-94 reduced verification

Y-94 implements the omission part of redesign phase 5. Failed-result
acceptance remains out of scope. The default chain is unchanged. Selected
ordinary graphs may omit any candidate gates, seats and landing gates.
Retained panel inputs and the final blocking route are rebuilt from the
retained seats. Admission stores skipped identities and unreviewed status.
Approval attention, including an automatic approval's policy lapse, derives
review status from the admitted plan. Empty landing uses the existing merge
preparation, persisted intent and canonical compare-and-swap.

## Test-audit gate

One scenario was added to g01_only_the_queue_lands.rs. Existing worker
refusal assertions are unchanged. No existing assertion block was removed
or weakened. This is the change gate, not a whole-suite sweep.

- G1: the same project completes its default full chain, then reduced and
  empty operator selections. The empty selection changes a protected path
  under auto policy. It waits for operator approval before queue landing.
  Sabotage: auto-approve the protected change, or run hidden configured gates.
- G3: each landing follows a target advance. Host git supplies the expected
  merge parents and tree. The persisted resolved intent names that merge.
  The empty selection has no gate execution or check. Sabotage: fabricate
  verification evidence or land a different tree.
- G8: the reduced panel omits its middle seat. The retained first seat blocks
  once; the passing final seat must carry that block and cause one repair.
  The selected plan rewires its predecessor. Skipped identities come from
  the fixture's declared omissions. The empty selection stays unreviewed
  despite a configured panel. Sabotage: retain the removed predecessor,
  discard the accumulated block, fabricate skipped passes or derive review
  status from the unchanged workflow.

Observers are trusted operator MCP, CLI JSON, the read-only store and host
canonical git. Expected node sets, execution counts, findings and protected
paths come from fixture declarations and the scripted model. Expected merge
parents and tree come from host git. The existing full-chain scenarios do
not exercise omitted nodes, a removed panel predecessor or an empty selected
panel on a reviewed workflow. These are the hardest omission cases: a gap
inside an accumulated blocking panel and no verification at all after a
concurrent target advance.

Test lines removed: 0. Binary lines removed: 32; the
full-node admission comparison is replaced by retained-definition matching.

Host e2e and sabotage runs are deferred to the approved landing gate by the
ticket's explicit instruction. No development daemon opened the construction
backlog. Validation passed: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings`. Clippy compiled all
e2e targets without running them.
