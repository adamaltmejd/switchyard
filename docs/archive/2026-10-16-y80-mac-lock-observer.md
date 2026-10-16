# Y80 Mac lock observer correction

2026-10-16. Operator full Mac CI `38926812012` on exact
`34f3159519d2bc65fa2f752b6328c65aeccd877c` passed 43/47 in 498.37 seconds.
The same exact source passed localhost 47/47 in 533.57 seconds and Linux
arm CI. All failed results remain evidence. These are supplied operator
observations, not runs performed in this worker box.

G4 concurrent readers failed parsing owner_fields[3] as an integer at line
789. The host shell's `/dev/fd/*` and `-ef` scan returned `holder=missing`
on Mac. That descriptor enumeration is not evidence of a lost product FD.
Remove the descriptor-number scan, its numeric assertion and the lock-path
argument used only by that scan. The real delegated exec, inherited-lock
child, FIFO acknowledgment/release, PID, exact box and exit-code observations
remain. The existing OS File::try_lock WouldBlock checks before and after
restart, successful lock after release and same host device/inode comparison
remain unchanged, as do all capacity, publication eligibility and checked
release assertions.

Test-audit gate: this is the existing G4/G5/G7 ownership/publication hard
case, with no extra machine, fixture, build, box or model turn. Premature
owner release or application before checked teardown remains the material
failure, covered by existing sabotage comments. The OS lock result, live
held child, host inode and scripted FIFO release are the external oracles;
a descriptor number is not. No product guarantee or Shown by claim changes.
No new helper, dependency, retry, deadline or concurrency change. The source
diff removes 10 test lines and adds 4; zero product lines change. No elapsed
savings or sabotage execution is claimed.

The two G8 Durable failures were Apple BuildKit `archive/tar: invalid tar
header` during image build before work. The operator is investigating the
runtime/context cause; G8 controls, assertions and historical source are
untouched, and this correction does not claim to fix those failures.
Prior archives and all failed results remain. Boxed fmt, locked all-target
clippy, build and diff checks passed before commit. No host/e2e or runtime
action ran here. Full combined-source Linux/Mac qualification remains with
the operator after the build-context cause is understood. Product code,
pins, construction state, `.yard` and installed binaries remain unchanged.
