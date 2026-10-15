# Y-155 reviewer observer callsite audit

The follow-up audit starts at 21d4342. Its capability filter already fixes
the gate3543 observer failure. ModelRequest::has_tool reads the request's
offered tools, not its prompt. Frozen source instructions therefore cannot
give a finding picker the reviewer's identity.

Rename the local counter to reviewer_opens. All nine calls use that same
capability filter: initial left/right/join counts, restarted left/right
counts, the first exact join, and revised left/right/join counts. There are
no other uses of this helper in the suite. Keep every expected count.

Test-audit Gate disposition: retain the existing assertion blocks.

- G2/G8/G13: exact historical inputs, committed selection settlement and
  fresh inputs after candidate changes remain in the existing hard case.
  Reviewer counts detect replay; store assertions separately count exact
  executions and selections. G2/G8 product controls remain unchanged.
- Sabotage remains replaying a reviewer or reusing a stale selection. The
  existing scenario's sabotage comment and exact assertions remain intact.
- No new scenario, machine, build, box or model turn is added. All refusal,
  contributor, raw finding and checked ownership assertions are retained.
- Expected counts and actor instructions come from the fixture. Offered
  publication capabilities distinguish the actor; the public store supplies
  independent execution, result and source identity evidence.

No test assertion or binary line is removed. No deliberately untested
behavior or product contract changes. The rename changes ten test lines.
Earlier dated audit files remain historical records.

Preserve the reported gate3543 failure: 44 passed, one failed, 370.59 seconds
at candidate 955a99a6fa9c88ff0528611f8cefb81d22d321a8. The terminal excerpt
remains in /yard/proof/y155-955a99a-terminal-landing-failure.txt. The operator's
raw y-155-gate3543.log and observer-count-diagnosis.json remain referenced at
/home/yard/validation/y111-operator-evidence-20261013/approved-y155-955a99-gate-capture/.
That host directory is not mounted in this worker. This audit does not
claim a new host run or independent verification of those retained files.

Worker validation passed: cargo fmt --check, cargo clippy --all-targets
--locked -- -D warnings, and cargo build --locked. Native e2e execution is
deferred by the operator to the sequential host landing gate after approval.
