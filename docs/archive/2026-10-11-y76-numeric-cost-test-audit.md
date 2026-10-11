# Y-76 numeric cost oracle repair

The operator reports focused host validation of exact head `779c4e6`:
exit 101, zero passed, one failed, 59 filtered, 27.28 seconds. Captured
interruption frames contain two assistant snapshots with id `msg_fake_3`,
each reporting input 1000, cache read 70, cache creation 30 and output 0.
Only init and two user frames accompany them. There is no result or later
nonzero output report. This establishes the explicit-zero partial observation
for that pinned interruption. The raw-frame oracle passed that phase; no
missed-update production bug was demonstrated.

The focused run passed reopening and reached final resumed completion.
Input 2200, output 200 and complete harness metadata agreed. Its failure
was numeric representation: stored cost JSON `0.0` compared unequal to
the transformed raw result's JSON `0` under `serde_json::Value` equality.
The later raw fresh/resumed estimate comparisons did not execute. These
facts are operator-provided host evidence, not a new worker runtime run.

## Test-audit disposition

Keep the complete four-phase common accounting scenario. Change the cost
portion of its record comparison to G7's numeric USD meaning. Convert the
external harness report and observed execution cost with `as_f64`, requiring
both to be numeric, before comparing the record. No missing value is
defaulted or converted to zero. The unreported-cost branch leaves the actual
record unchanged and still requires exact null and unknown/unreported fields.
Only cost representation is normalized; token values and metadata are not.

This block serves G7's optional estimate and explicit-zero observation clause.
Its expected amount still comes from the real result frame after the host's
documented measurement transformation. The observer is execution JSON.
The existing test comment names the sabotage: default absent cost to zero.
Discarding explicit zero also fails numeric conversion or record equality.
The successful zero report is the control for the missing-cost observation.
No lexical integer/float JSON representation is part of the guarantee.

The hardest case remains final resumed success after failed and interrupted
turns and reopening. No earlier block establishes this explicit-zero cost
with complete metadata on that final execution. No duplicate scenario or
assertion was added. All token, provenance, completeness, null-cost,
interrupted raw-frame, reopening, terminal-replacement and estimate checks
remain. The separate `cost.is_number()` assertion is replaced by numeric
conversion of both the expected report and observed value, which also fails
for null. No promise or shown-by clause is removed. The binding contract
and production source are unchanged.

The operator's numeric oracle finding is addressed. Five test source lines
were removed and nine added; zero binary lines were removed. No acceptance
of the subsequent estimate checks is claimed.

## Validation

`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `cargo build --locked` passed in the worker. No host e2e, sabotage run,
daemon, public model call or construction configuration edit occurred.
The repaired exact head still requires operator focused/full host validation
and fresh review before approval and landing.
