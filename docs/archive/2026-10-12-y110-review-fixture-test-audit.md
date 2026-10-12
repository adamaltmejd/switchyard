# Y-110 review fixture corrections

Review blocked af034ef with three findings. All three are corrected in
the existing held-preparation scenarios. Production code is unchanged.
Earlier host failures and the stalled ownership fixture result remain
recorded in the preceding dated archives; this correction does not claim
that those runs passed or reached their later blocks.

## Dispositions

P1: the successful and failed preparation callers now supply distinct
Machine names. Their process-local root directories, daemon state, sockets
and FIFOs no longer collide under default parallel execution. Reverting
the names to one shared root exposes the race when both run concurrently.

P2: remove both test-side deletions of Yard-managed attempt clones. The
fixture-owned Git wrapper still delegates unchanged argv to real Git,
records its native exit, and reports 73 only after native success. Before
reporting that failure it removes only the clone destination it just
created, taken from that real invocation's final argument. Cleanup failure
propagates instead of reporting the expected 73. This keeps fault creation
and cleanup inside the same tool fixture, before child completion and
ownership release. The later deliberate starts use only the CLI. It does
not claim recovery of an arbitrary partial checkout. G9's shown-by text
now describes CLI starts without a fixture-workaround condition.

P3: the read-only execution identity query now orders by id. Its expected
identities remain fixture execution order, independent of SQLite row order.

## Test-audit gate

1. G4/G5 establish retained capacity until supported ownership proof.
   G9's preparation-stop shown-by covers successful and failed real Git
   completion, stop and no-stop native startup refusals, a surviving startup
   owner, successful returned-owner teardown, and deliberate CLI starts.
   No guarantee or assertion is removed or weakened.
2. The existing comments retain the binary sabotage: propagate preparation
   Err before honoring stop, ignore preparation stop checks, release at
   acceptance, discard the returned LiveBox, or release a failed startup
   on box absence alone. Fixture root collision is an additional test
   determinism defect, corrected without adding a scenario.
3. Successful preparation cannot expose the error handoff. Startup adds
   the distinct observer of actual box ownership before a recorded handle.
   The hardest case still holds the inherited host lock after real box
   disappearance. All original clone controls, startup controls and owner
   assertions stay in the existing scenarios. No twin is added.
4. Expected exits come from the native Git result and fixture exit 73;
   identities from fixture ticket/execution order; ownership from host PID,
   real Pinfold inventory and kernel locks; reservation and outcomes from
   read-only store observation; acknowledgements from CLI history; turns
   from the host model. The spec supplies stopped:unchanged. Explicit query
   ordering merely makes the fixture's identity comparison deterministic.

Seven test source lines are removed or replaced, and zero binary source
lines are removed. The two private-state cleanup calls and their comment
are removed; all assertion blocks remain. This is a scoped change gate,
not a whole-suite sweep.

## Validation

The exact updated Git shell wrapper was extracted, re-read and passed
sh -n after static placeholder expansion. cargo fmt --check and
cargo clippy --all-targets --locked -- -D warnings passed.

No e2e suite, concurrent scenario or sabotage run was performed here,
as instructed. Static checks do not prove real host execution. The host
landing gate must run both preparation scenarios concurrently and complete
the failed-preparation scenario's retained-owner, checked-release and
deliberate-start assertions before approval.
