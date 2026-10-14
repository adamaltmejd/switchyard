# Y-65 and landed Y-152 fixture integration

The candidate merges canonical e2fe045704160871b3ca49706e0214b7be050e10.
Its g08_durable_package.rs conflict is resolved with both changes retained.
Y-152's shared dependency target, Yard package clean, locked rebuild and copy
to each named executable remain. Y-65's historical runner override, sealed
debugger launcher and three independently runnable continuation scenarios
remain unchanged.

Y-152's original serial-builder premise no longer applies. The historical
package scenario and debugger scenario can build concurrently. One local
fixture mutex now covers the entire clean/build/copy sequence in their shared
helper. Cargo's per-command locks alone leave the intervals between commands
unprotected. The guard starts after independent source and asset preparation
and drops when the helper returns, before any scenario runtime. The debugger
scenarios retain their shared OnceLock build. No test-thread setting, runtime
lock, production cache or execution lifecycle is added.

## Test-audit Gate

Disposition: retain every G2/G5 continuation and G8 package assertion,
refusal control and scenario. There is no guarantee or acceptance narrowing.
The only added assertion is Y-152's clean-command success check, a fixture
setup check. Expected success comes from the real Cargo process exit status.
No new behavioral assertion block or twin is introduced. The mutex itself
adds no assertion or test-only path to Yard.

The existing G8 sabotage of using the wrong compiled package still exercises
the asset identity checks. Cleaning prevents stale archived timestamps from
reusing a different seal; locking through the executable copy prevents another
builder from replacing it between build and copy. The existing historical
admission and adoption observations remain the controls. Y-65's held unsafe
read, changed input, corrupt checkpoint, exclusivity and usage-increment
oracles and sabotage comments remain as recorded in its earlier audit.

Test lines removed relative to Y-65: one target argument replaced.
Binary lines removed: zero. The earlier Y-152 archive remains dated evidence
of its measured builds, not a claim about this integrated candidate.

## Validation

Boxed cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings
and cargo build --locked passed on the integrated candidate.
No host e2e scenario was run in this box. Prior isolated Y-65 host runs do
not qualify the integrated fixture. A fresh candidate review and affected
actual host scenarios remain operator gates before full landing.
