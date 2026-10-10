# Y-85 future-admission test repair

The operator read the full diff of candidate
29914464057985a502ec75b4867e76250d2945a3 and identified a lifecycle race.
Abandoning an attempt leaves its ticket ready for admission. The scheduler
could start another attempt during the later fixture observations.

## Dispositions

| File | Row and clause | Disposition |
|---|---|---|
| g10_queue_one_at_a_time.rs | G2/G6 frozen inputs through repair and restart, with future v2 admission; G10 conflict repair | Park Y-3 through the CLI before abandoning its attempt. Retain all counter, provenance, trusted staging, image and reviewer assertions. |
| g11_only_sync_changes_yard.rs | G11 only operator sync supplies trusted configuration; G2 future admission | Park Y-2 through the CLI before abandoning its attempt. Retain the future gate and repository divergence assertions. |

## Gate answers

No assertion block changes. The future admissions remain positive controls
for new resolver definitions. Parking through the supported CLI prevents
readmission while abandonment ends the attempt. The subsequent observations
therefore refer to the fixture executions already requested.

Late workflow, gate, reviewer, image or trusted staging reads still defeat
the G10 frozen-input controls. Reusing old admission inputs for future work
still defeats the future v2 controls in G10 and G11. Those sabotages remain
named in the test comments. G10 combines repair, changed trusted inputs and
restart; G11 exercises the operator sync boundary. Their observations cover
different mechanisms. Literal fixture counters and definitions, host Git
object identities and model requests remain the external expected values.

All existing assertion blocks, full provenance equality, policy withdrawal
controls and legacy unknown semantics remain intact. No binary change,
sleep, retry, model suppression or barrier was added. The repair removes
zero test lines and zero binary lines.

No host run is claimed. The operator will run real host e2e on the repaired
exact candidate after the source and test-audit gate. Candidate validation
uses cargo fmt --check and locked all-targets clippy with warnings denied.
