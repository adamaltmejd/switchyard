# Integration recovery observer, 2026-10-15

CI run 38722738619, arm64 job 116381056435, tested source
0e4942515c866c0438088db7b4411b9122c5a479 with Pinfold 0.2.3.
The suite failed 1 of 51 scenarios in 301.33 seconds. The retained
integration scenario resumed immediately after daemon restart. The real
operator MCP refused with workspace_owned, naming execution 5 and the
integrate workspace. This result remains a failure.

Startup reconciliation may retain an owner whose absence is not yet proved.
The scheduler later reaps that owner. The fixture now reads history until
its exact crashed integration execution has both released and ended, then
requests continuation once. No retry, sleep, deadline, assertion, runtime,
production code or control changed. A permanent owner still fails the wait.
This addresses the premature continuation request; it does not identify
why the original ownership probe retained the owner.

Test-audit Gate: G2/G9 protect retained partial work, immutable integration
inputs and approval of the integrated candidate only. Their existing held
real model request and host clone checks prove the crash state. Existing
host Git and fixture file contents supply all expected results. No new
machine, box, historical build, model turn or assertion was added. The
existing sabotage comment still applies: discarding the retained clone or
selecting an input candidate fails the existing assertions. The history
boundary removes an incidental scheduler race from those observations.

The daemon-lock restart refusal tracked by Y147 and ordinary stopped-worker
case tracked by Y148 remain unresolved and are not changed here.
