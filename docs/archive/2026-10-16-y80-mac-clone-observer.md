# Y80 Mac clone observer correction

2026-10-16. Follows the G4 observer repair. Operator Mac full CI
`38926812012` on exact `34f3159519d2bc65fa2f752b6328c65aeccd877c` passed
43/47 in 498.37 seconds; localhost on the same source passed 47/47 in
533.57 seconds and Linux arm CI passed. The operator retains all failed
results. No host execution was performed in this worker box.

G13 selected cleanup failed at line 767 comparing guest `.git` inode
outputs 45, 47, 47 across separate role boxes. Apple VM guest inode values
belong to each VM; comparing them does not establish host clone identity.
Remove that comparison and its orphaned guest stat commands and result
collection. Retain the cleanup source-write and operator-only grant probe.
Instead, after repair read `chain.txt` from the original host clone and
require the fixture's exact `repaired\n` bytes. The original host `.git`
inode comparison, held cleanup bytes, exclusive workspace refusal, ordered
checked release, report-only cleanup, selected settings, commit ancestry,
candidate verification and native landing assertions remain unchanged.
The G9 Shown by wording now names the host clone Git inode accurately.

Test-audit gate, G2/G9/G13: the material failure is repair using a private
clone instead of the one exclusively shared by sequential roles. The
existing one-machine planner/three-role hard case already supplies the
original host path and distinct implemented/cleaned/repaired bytes. A
private repair clone leaves the original host chain.txt cleaned, so the
new assertion fails even if that private clone has valid ancestry and
publishes a candidate. The sabotage comment now names this external oracle.
Host filesystem bytes and inode, fixture literals and Git ancestry provide
expected values independently of Yard's workspace claims. No same-clone
guarantee is weakened. Existing allowed and refusal controls remain.
The test diff removes 26 lines and adds 14; zero product lines change.
There is no new helper, dependency, machine, historical build, box, model
turn, retry, deadline or concurrency change. No sabotage was run here and
no measured runtime saving is claimed.

The two G8 failures remain Apple BuildKit `archive/tar: invalid tar header`
during image build before work. The runtime/context minimal reproduction
is operator work; no G8 assertion, control or historical source changes.
The original Mac legacy startup failure and all earlier source-bound
evidence remain separately preserved. Boxed fmt, locked all-target clippy,
build and diff checks passed before commit. No e2e or host/runtime action
ran here. The operator will qualify the combined source on Linux and Mac
once the build-context cause is understood. Product code, Pinfold/runtime
pins, `.yard`, construction state and installed binaries remain unchanged.
No full qualification, release approval or switchover is claimed.
