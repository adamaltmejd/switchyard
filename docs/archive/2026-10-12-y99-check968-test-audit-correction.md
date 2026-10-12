# Y-99 check968 assertion correction, 2026-10-12

Review check968 P3 is confirmed. The predecessor_read and predecessor_write
probes passed a host checkpoint path into a box where that path was never
mounted. Those probes failed with either exclusive or shared harness state.
They did not discriminate the claimed isolation sabotage. Delete both probes,
the prior-checkpoint fixture file and shell argument plumbing, and their
assertion iterations. Retain the proof-write assertion directly.

This record corrects the evidence claims in the earlier Y-99 dated assessment.
That file remains unchanged history. Production code, ARCHITECTURE.md, G13,
all mounts and the execution ownership boundary remain unchanged. No scenario
or twin is added. The existing packaged-child scenario remains the survivor.

## Surviving assertions and sabotage

- G13 exclusive writable state: predecessor_visible probes the actual
  `/yard/state/audit-checkpoints/owned` path before each Pi starts. The first
  child then creates its real checkpoint directory, and real box exec plants
  fixture bytes there. The replacement is held before invoking its child.
  Mounting the predecessor state at the replacement's `/yard/state` exposes
  that marker and fails the fresh-state assertion. Unlike the deleted probes,
  this observation uses the mount whose exclusivity is promised.
- G13 retained checkpoint: the host reads the fixture checkpoint before the
  edit and while the replacement is held. Its continued presence prevents
  cleanup from explaining absence in the replacement mount. Changed bytes
  fail the immutability observation. This supports the mounted-state oracle;
  it is not an independent denial-of-access test. Together, the retained
  checkpoint and its absence from the actual replacement state show isolation
  of the predecessor checkpoint without weakening the read/modify promise.
- G13 no resume and retained ownership identity: both execution rows have
  null resumed and the fixture plan's implementer node. Selecting a predecessor
  session or losing that binding fails these observations. Execution rows
  remain the authority; no new ownership record is introduced.
- G13 frozen proof: real box exec confirms `/yard/proof` is a directory and
  refuses a write there. Omitting that mount fails presence; making it writable
  fails write refusal. Existing source and trusted-input write refusals retain
  their original mount sabotage and observations.
- G13 successful controls: the replacement's real packaged child publishes
  the fixture report, teardown is checked, and regular scratch/checkpoint
  writes succeed. These controls prevent a broken box or an unusable writable
  state directory from satisfying isolation. Regular declared source/trusted
  inputs remain successful controls for the original packaging refusals.
- G13 host link safety: real box exec plants a checkpoint link to a host
  sentinel directory. After history observes whole-attempt cleanup, the host
  sentinel bytes remain. Following guest links during recursive cleanup fails
  this oracle. Fresh staging never enters predecessor checkpoint paths.
- G8/G12: all original package preflight, declared-read, publication and secret
  assertions are retained, including successful controls in the same project.

The hardest case remains the edit on the same attempt after the first real
packaged child publishes and before outer Pi exits. Both model holds remain;
there are no sleeps, retries, timeout changes or added runtime seams. Expected
checkpoint, report and sentinel content comes from the fixture. Mount probes
run through real pinfold exec; the retained checkpoint and sentinel are host
observations. Null resume and admitted node come from G13 and the fixture plan.

G13's seat/gate snapshot scenario has no audit successor checkpoint. G14's
writable repair tests safe models-file replacement, and its proof-link scenario
refuses a malformed candidate. Neither duplicates the actual audit-state mount
observation. The earlier twin dispositions remain valid. Removing ineffective
host-path probes leaves every original Y-99 promise and successful control.

Test lines removed: 19; test lines added: 9. Binary lines removed: zero.
The proof-write loop is simplified without changing its assertion.

## Validation

Boxed `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `cargo build --locked` pass. Host e2e remains the operator-owned landing
gate and is not run in the candidate box. No runtime or sabotage pass is
claimed from compilation. Y-93 remains the final full-scope integration.
