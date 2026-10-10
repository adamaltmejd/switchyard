# Y-82 operator validation — 2026-10-10

Y-82 landed named branch sync and explicit project-default selection.
The frozen construction daemon and executable were retained. This is
construction evidence, not release completion or publication approval.

## Candidate correction

The first candidate was `c54deb2e194d411ca1c09b096e86fd19967f5ad4`.
Its three G11 host scenarios passed in 29.19 seconds. Operator review
found that unconditional canonical locking blocked ordinary ticket edits
and proposal decisions behind landing git children.

The existing G13 host scenario reproduced this through the real CLI.
External process observation showed held `merge-tree` and waiting
`ticket edit Y-2` together. Git reached its configured deadline. After
the failed child's reader died, the operator drained the isolated fixture's
release pipe so the assertion could report and the fixture could clean up.
The scenario failed on the daemon's `git merge-tree timed out` error at
G13 line 335, in 355.06 seconds. This intervention is failure evidence,
not a passing test. The isolated fixture daemon exited.

Approval attention 129 was rejected with exact head and proof identity.
Repair execution 1107 narrowed locks to configuration-dependent changes,
preserved ordinary edits and proposal decisions, separated default
selection from consume, and added held-intent coverage. G3 and G13
assertions were preserved.

## Verified repair and landing

The repaired head was `1e540d2510b83adf4c8afcc7480e1f913642edda`.
Its proof was
`e75af617033cb6aded57f2f78909efcc97db09166166698eb75c1bdc636268a7`.
Operator review read the implementation, assertion and binding-doc changes.
Read-only implementation and test-audit reviews found no further defect
in the repair. Candidate fmt, locked all-targets clippy and build passed.

An isolated checkout of that exact head passed all 40 real host scenarios
in 131.27 seconds, without operator intervention. This included G3,
G13, all four G11 scenarios and the direct API credential fixture.
DeepSeek review check 587 passed with one P3 assertion finding.

Approval 68 bound the repaired head and proof. Landing execution 1112
passed fmt, clippy, build and all 40 host scenarios in 147.45 seconds.
`landing.recorded` was event 3857; ticket completion was 3858.
Cleanup execution 1117 ended `removed` at 3861. Sync consumed the
landed head at 3862.

Logs retained for this run:

- `/tmp/switchyard-y82-focused-host.log`
- `/tmp/switchyard-y82-g13-host.log`
- `/tmp/switchyard-y82-repaired-host.log`

## Required audit carry-in

Check 587 identified a missing-Dockerfile assertion against message prose.
It remains unresolved. Parked Y-79 revision 5 now requires a spec-defined
machine reason and committed input identity through the supported MCP/API
observer. It must preserve the unchanged-ref observation and restored-input
positive control. The earlier G7 watch-observation carry-in also remains.
Neither clause is waived by the passing suite.

Y-56 can now proceed. Y-81 remains parked. Required arm64, Mac, final
cleanup, migration, artifact and release checks remain outstanding.
Nothing was pushed, published, installed or switched. The operational
0.0.3 executable still hashes to
`05ae25832964703c81525e7c5f5be7a8148954517cfddf69ef9846953bf1ebb2`.
