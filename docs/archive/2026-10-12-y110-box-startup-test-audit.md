# Y-110 box-startup preparation-stop repair

The operator reports that candidate 3f0a9f9 passed both clone controls.
The precise error-first mutation failed directly at G4:941.
Review 2234/check 1161 passed and noted startup at P3.
The operator then reproduced the startup gap through the real CLI.
Private evidence is under
/home/yard/validation/y110-3f0a9f-review-20261012, in the
operator-box-startup-stop-probe patch, log and result JSON.
That failed host probe is operator evidence, not worker reproduction.

An acknowledged stop preceded real Pinfold's native mount refusal.
The execution ended error with red:error before any handle or worker turn.
The existing contract includes this preparation handoff.
The earlier archive describes the previous candidate only.

## Repair

Only up_worker's error arm uses the existing preparation-result stop check.
It retains the original failure diagnostic on unchanged/stopped:unchanged.
up_worker still revokes grants and forgets the image on native failure.
A no-stop error still propagates. Worker errors keep their existing path.

The successful arm retains its returned LiveBox. run_harness consumes any
pending stop before the turn; its existing teardown awaits the controller.
No returned owner is discarded at a preparation stop check.
Both arms remain under the single capacity::effect lock. Existing real
inventory and host-lock proof decide release. A failed startup can still
leave an owner; stopped attention does not replace red:ownership or release
its reservation. No lifecycle, runtime flag or ownership proof is added.

## Test-audit gate

Retain both existing held-clone scenarios and every original assertion,
including the unstopped wrapper failure and successful restart controls.
Retain all unknown-owner coverage. Extend the existing failed-preparation
scenario in the same project, rather than add a startup call-site twin.
Its fixture alone allows four attempt lanes to admit a fourth ticket
beside three approvals. The execution cap stays at one.

The four gate answers are:

1. G4/G5 require retained capacity and supported ownership proof. The Work
   contract requires acknowledged preparation stop to end unchanged with
   no worker turn. G9's shown-by now includes failed and successful startup
   handoffs and retained ownership. No contract is narrowed.
2. Sabotage: propagate up_worker Err before honoring the pending stop.
   Discard a successful LiveBox at a preparation stop check. Or release
   a failed startup on box absence alone, ignoring its inherited host lock.
   The fixture comment names these source changes.
3. Clone completion does not expose a startup owner. The extension holds
   real startup before native invocation, then makes only its own mount
   unavailable. It tests stop and no-stop native refusals on the same ticket.
   The native reply is captured unchanged and copied to stderr, and the
   wrapper exits with the native code. Yard observes EOF after that child
   finishes, with the original native diagnostic, without a synthetic frame.

   The hardest ownership case loses an actual native ready reply. The
   unchanged spec and argv reach real Pinfold. Its real box and controller
   stay up on fixture-held stdin, while the wrapper exits and Yard sees EOF.
   Actual inventory and the inherited host lock prove the surviving owner.
   The stopped row remains unchanged/reserved with null handle and
   red:ownership. After native stdin closes, actual box absence still cannot
   release capacity: a drain child retains the inherited lock. Two held
   successful real inventory replies prove a release probe ran in that
   state. Only host-owner exit permits checked release. The operator then
   answers both existing attention exits through attempt start.

   A successful startup with pending stop holds real stdin EOF during normal
   teardown. The actual box, recorded handle, running row and reservation
   remain, with no model turn. Native teardown, box absence and lock freedom
   precede checked release. A deliberate final start reaches approval.
4. Expected identities come from fixture ticket/execution order. PID and
   kill -0 observe the held child. Pinfold's actual native failure reply is
   the complete external diagnostic expected in the store. Its actual ready
   reply proves startup, its inventory proves box presence and absence,
   and try_lock observes the inherited host owner. CLI history supplies
   acknowledged stop, exact attention and release events; the read-only
   store supplies handle, outcome and reservation. The host model supplies
   worker-turn observations. Spec supplies unchanged/stopped:unchanged and
   the ordinary unstopped red:error outcome.

No assertion or scenario was removed. Relative to 3f0a9f9, the diff replaces
one test source line (fixture lane count) and three binary source lines
(startup propagation and the helper comment). This is a scoped test-change
gate, not a whole-suite sweep. There is no weakened clause for an operator
decision and no new follow-up required for this handoff.

## Validation

cargo fmt --check and cargo clippy --all-targets --locked -- -D warnings
passed. The two multiline shell fixtures also passed sh -n after static
placeholder expansion. No fixture runtime was simulated.

No e2e or sabotage run was performed here. The host container runtime
remains the landing gate, as instructed. In particular, compilation and
shell syntax checks do not claim that the newly extended host scenario
passed.
