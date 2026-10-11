# Y-94 landing gate image precondition repair

The operator reported a host landing gate failure on merged ref
ba8a271d1f0d3a9ffe65a340c534e39c0b3ada56, targeting
fe6543410ef1ee336d18fcd8f612a0bb5382a4e6. The suite reported 62 passed
and one failed in 304.35 seconds. The failed scenario was G8
`a_gate_errors_start_reruns_that_gate`.

Its initial planner never reached the scripted model. Pinfold image build
reported a cache hit for c483175b78b397664d637de5345e5e23defd7fd970bd006f7ec56162d92f7f47,
then failed during commit/copy because that image was not known to the
container runtime. The daemon raised red before the scenario could prove
candidate retry, landing-child infrastructure error or abandonment. This
is a fixture precondition failure, not a pass for those phases. The supplied
log excerpt does not independently identify the Y-94 G1 scenario's result.

## Boundary and repair

Machine wrappers record built image names. Machine::drop kills that fixture's
daemon, prunes boxes and retires its recorded names. All fixtures share
Pinfold state and the host container runtime. Project roots give distinct
image names, but the unchanged scaffold Dockerfile gives identical image
contents. Daemon image-build locking protects one daemon, not other fixture
machines' builds and retirement. The supplied log does not establish which
operation removed the cache entry or prove an upstream defect.

The Y-77 image-isolation audit records the same shared-content exposure and
a fixture-only mitigation. This repair gives only the failed G8 fixture a
Dockerfile label naming its machine root, including process ID and scenario
name. The fixture commits and syncs that content before admitting its planner.
Its final image configuration differs from other fixtures' scaffold images.
Production image building, cleanup, schema and worker grants are unchanged.
No repository .yard file changes. No retry, sleep, simulated runtime or
weakened assertion is introduced. Fresh host validation must establish
whether this removes the observed precondition failure.

## Test-audit Gate

1. G8's shown-by clause covers candidate and landing-child box-up errors
   with successful controls. It now records the machine-specific image.
   All existing assertions, including the allowed same-head candidate retry,
   selected landing order, retained approval, exact merged input, absent
   judgment/repair on infrastructure failure and abandoned host gate, remain.
   The Y-94 G1 full/reduced/empty scenario is unchanged.
2. Existing sabotage comments still name treating gate errors as worker
   stops, routing infrastructure errors as failed judgments, requiring full
   workflow translation on retry, and recording abandoned gate checks. Those
   mutations still fail the original observations once the fixture runs.
3. This adds no scenario or assertion twin. The scenario still tests the
   harder sequence: refused candidate bring-up, successful same-head retry,
   target advance, refused landing child, successful requeue under retained
   approval, then abandonment of a held host gate in the same project.
4. Expected identities and gate ordering come from declared fixture inputs,
   expected refusal and abandonment come from the fixture's real wrapper and
   held host command, and merge parents come from host git. CLI JSON and the
   read-only store remain the external observers. A unique image label is
   setup, not an assertion or an oracle copied from the scheduler.

Test lines removed: 0. Binary lines removed: 0. No guarantee is weakened.
This is the test-audit change gate, not a whole-suite sweep. No focused/full
host suite or sabotage run is performed in this worker. The failed host
result above is operator-supplied evidence; it does not validate this repair.
Static validation passed: `cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings`. Fresh exact-head host gates and published review
remain required before approval.
