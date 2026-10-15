# Y-155 landing observer correction

The real host landing gate failed on merged candidate
955a99a6fa9c88ff0528611f8cefb81d22d321a8 against target
9e4788b754d2c47d43704c5f9de67630b69144f0. It completed 44 passing scenarios
and one failure in 370.59 seconds. The join scenario reached line 851:
opens("Exact join.") observed 2 instead of the required 1. Its preceding
store assertion found exactly one join execution. The new held-owner,
restart, exact reconciliation and immutable-result assertions had completed.

The request counter searched all Graph prompts for reviewer instructions.
The finding picker exposes its exact sources, including their frozen node
instructions, so its own opening request also contains "Exact join.".
This is a mixed-role observer, not evidence of a replayed review. Other
reviewer counters could likewise count picker context at later revisions.

Gate disposition: retain the existing hard case and expected counts; narrow
the observer to native requests with yard_publish_review capability. No
assertion or product code is removed, and no scenario is added.

- G2/G5/G8/G13: the existing scenario protects exact source identity and
  restart settlement without duplicate actors. A repeated reviewer still
  fails the original count and store assertions. The fixture comment names
  duplicate/stale selection and ownership sabotage already.
- This costs no machine, build, box or model turn. It fixes the existing
  observer rather than replacing the hard held-owner case or relaxing counts.
- Expected reviewer instructions and count come from the fixture. Native
  publication capabilities distinguish real reviewer turns from picker turns;
  public store executions independently establish one exact join. This is
  observation of who acted, not a schema-shape assertion.
- All existing refusal controls, held-owner evidence, immutable rows, raw
  source preservation and candidate-change controls remain unchanged.

Test diff: 3 added lines, zero removed lines. Binary diff: zero lines.
The living representative Shown by clauses still describe the same scenario.
The reported terminal failure is retained in candidate proof. Worker gates
are cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings,
and cargo build --locked. Native rerun remains the sequential real host gate.
