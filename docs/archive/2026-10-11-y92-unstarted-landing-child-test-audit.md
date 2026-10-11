# Y-92 unstarted landing-child repair, 2026-10-11

## Exact-head evidence

The operator's clean full suite at dd877175e290ec5106a77eafdaed40cc06f023b8
ended 71 passed, 1 failed, exit 101, in 702.08 seconds. Focused recovery,
gate-error and package runs and current review 923 had passed. These greens
do not establish full-suite or actual-landing acceptance. Canonical remains
e4ce7d57e5537679f78125a00f46a3ab4dc7d725.

a_gate_errors_start_reruns_that_gate expected red:landing at line 879 but
received red:ownership for landing execution 6, seq 41, at
2026-10-11T07:04:07.841Z. Before fixture Drop, external read-only store and
audit snapshots showed child 7 admitted at 07:03:37.837Z, node
landing-gate-check, parent 6 and merged inputs. Parent 6 ended error with
reserved=1 and unknown ownership. Child 7 stayed running, reserved=0,
teardown=0, result=NULL and handle=NULL; its controller never ran. A later
free parent lock does not prove freedom at the refused release.

Read-only kernel observers separately showed podman ps waiters blocked on
overlay-images/images.lock inode 163029 while another package fixture's
podman run held it. Verified waiter working directories were g8-killed and
g8-durable-bound, with observed ages 10 to 55 seconds. Another listing was
observed at ages 10 to 29 seconds during the admission/release window with
a fcntl_setlk thread. Its fixture association was not verified before exit.
This establishes image-store contention in this run. It does not establish
the exact failed probe's cause or the historical 1751 or Y-95 root cause.
The operator retains the complete log and snapshots outside this box.
These facts are reviewable upstream report material; no report was published.

The published-seat recovery and both package cases passed in that full run.
The external observer showed execution 4 release at seq 33, exact ownership
resolution at 34, interrupted end at 35 and approval at 36. That recovery
guard and every original assertion are retained.

## Admission repair

queue::land previously admitted the child inside capacity::effect and used
an early question-mark return on the enclosing effect. The work committed
the child before checked parent release. A release error discarded the
returned identity without ever starting or ending the child controller.

The caller now retains that committed child identity across the enclosing
effect. If the effect fails, the existing daemon transaction ends only that
still-running, unstarted child as interrupted. Its admitted binding remains
unchanged. Existing execution end records interrupted infrastructure with
empty outputs and no judgment or route. No checked teardown, reservation,
publication or passing check is fabricated. Parent release and its error
continue unchanged, retaining capacity and ownership attention until genuine
proof. No owner, lifecycle, eligibility predicate or dependency is added.
G5/G8 and their shown-by clauses state this boundary in the same commit.

## Test-audit Gate

Keep and extend a_gate_errors_start_reruns_that_gate. Its original gate-error,
same-head retry, reason, no-judgment/no-repair, merged identities, approval,
landing and abandonment assertions remain unchanged. Add no test scenario
entry; the suite still has 72 tests. The extension runs in the same real
fixture project after the original controls and abandoned-attempt cleanup.

1. G5/G8 now name admission followed by parent ownership refusal before
   child controller start. The child must end without acting or gaining an
   eligible judgment; the parent must retain its capacity and refusal.
2. Sabotage: restore the early return that loses the admitted child identity.
   The exact child-ended history wait and terminal-state assertions fail;
   the running orphan also blocks the same-approval successful control.
   Releasing capacity without checked proof fails the retained-parent block.
   The test comment names the sabotage.
3. Existing box-up failure runs a child controller and then fails its launch.
   It cannot catch a child lost before controller start. G4's capacity waits
   and G5's restarted workers cannot expose this landing admission boundary.
   The new held listing reply forces this hardest admission/release edge;
   no overload, clock or timeout is used as evidence.
4. The real pinfold PATH wrapper delegates the actual listing, preserving
   stdout, stderr and exit status, then holds its completion on fixture FIFOs.
   Each real call gets one handshake. Preparation replies are released until
   the read-only external store shows the admitted child. The fixture then
   opens the parent's actual lock read-only and acquires a real external
   lock before returning the listing reply. This makes checked host ownership
   fail deterministically, without a fake pinfold response or private write.

The expected identities come from external read-only store observation and
must agree with status-history events and supported attempt show. Exact
merged base/head are checked against the fixture's current git target after
its trusted reconfiguration and canonical landing ref. Proof agrees with
the approved input.
Child binding equality before and after the control establishes preservation,
not a copied Yard calculation. The expected empty outputs, missing judgment,
route, handle, teardown and eligibility follow G5/G8. External lock ownership
lasts through the retained reservation, exact open ownership item, no
successor and canonical-unchanged assertions. After unlocking, history must
name that parent's checked release. Supported explicit start answers resolve
its existing ownership and controller-error items individually; nothing
automatically clears unrelated attention. A real host gate then passes and
lands under the same approval, with the original interrupted binding intact.

Keep a_seat_killed_after_publishing_has_published and its full recovery,
held-child refusal, second-publication, quick-ticket, no-early-route,
same-result and check controls. Review 923's P3 selector finding is repaired:
the wrapper records its PID by the real box argument and delegates every
real Pi invocation. After native registration and publication are held, the
fixture selects the exact review handle from the read-only store. That
wrapper retains its inherited effect lock on a FIFO after the real child
ends during restart. The pre-restart PID and lock probes and post-restart
ownership refusal still prove the same owner. No reviewer prompt wording
selects the hold. New recorded-box equality is an external wrapper/store
identity agreement. No assertion is removed or weakened. Review 923's
separate generic missing-path audit_failed diagnostic remains uncorrected.

Test setup lines removed replace the old prompt selector and its shared
owner-said FIFO with per-box PID records and an exact selected handle.
Production lines removed replace the admission early return. No guarantee
assertion block or control is deleted. No runtime stub, harness substitution,
mock frame, private launch-input write, sleep, retry, binary flag, public
endpoint, deadline adjustment or thread-cap adjustment is introduced.
Diff counts: 16 test setup lines removed, 216 added; 3 production lines
removed, 33 added. Removed assertion blocks: zero.

## Validation and remaining work

Candidate cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings,
cargo build --locked and git diff --check passed. A source comparison proved
all 15 guarantee rows remain and rows outside G5/G8 are byte-identical.
No e2e, image build, host package installation, host Node/client
or boxed controller run is performed here. Package source, pins, closure,
native bridge, CI repair, Y-97/Y-98 controls, all 15 guarantee rows, operator
configuration and old archive bytes remain intact. Repository .yard is not
edited. No new active ticket is created.

Fresh changed-head source/test audit, focused recovery and gate-error capture,
both package cases, full 72 host suite, published review and successful actual
landing remain required. This repair does not prove runtime contention is
fixed or diagnose the original 1751 gate-error failure. If runtime delays
still prevent validation, retain and report the concrete external blocker.
Y-93, parent Y-64 and release through Y-80 remain required; Y-81 stays deferred.
