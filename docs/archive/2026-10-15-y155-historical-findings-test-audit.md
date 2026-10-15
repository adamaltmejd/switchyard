# Y-155 historical finding observer repair

The operator reported gate3596 at exact candidate
f93181072fb36cf1410e830fbe347be19c342aee: 47 passed, one failed in 427.41
seconds. G04 line 923 compared all attempt findings to the revision-one
snapshot after revision two began. Only the right reviewer was held. The
new left reviewer had legitimately published findings 6, 7 and 8. Original
findings 1, 2, 3 and 5 retained all their field values.

Restrict both snapshot and equality query to the original plan revision.
Keep the full rows, ordering and expected count. This compares the exact
historical boundary and still detects deletion, mutation or extra findings
at that boundary. New revision publication remains concurrent.

Test-audit Gate disposition: retain the historical immutability block.

- G2/G8/G13: raw finding identity and values must survive revision and
  selection. Rewriting old evidence would corrupt inspectable provenance.
- Sabotage is erasing or rewriting old findings during revision, named in
  the block's comment. Complete original-row equality still detects it.
- Extend the existing hard case. Add no machine, build, box, model turn,
  scenario, hold, barrier, retry or sleep. No product behavior changes.
- Expected values are the full public-store rows captured before revision.
  The fixture's admitted revision one supplies the historical boundary.

Audit of the remaining scenario tail found no equivalent growing-set
comparison. The original join query has an explicit revision-one boundary.
Later source queries bind revision and head. Selector history comparisons
check expected growth and preserve full earlier rows. After native landing,
raw findings are looked up by each original finding ID and compared in full;
that correctly permits newer findings. The final original review execution
comparison already binds revision one. Keep all these assertions and native
landing, stale-refusal, duplicate-contributor and owned-restart observations.

Test lines replaced: two. Test assertions removed: zero. Binary lines removed:
zero. Product controls and representative Shown by clauses remain unchanged.

The operator retains raw log, terminal, store and
historical-findings-observer-diagnosis.json under
/home/yard/validation/y111-operator-evidence-20261013/approved-y155-f93181-gate-capture/.
That directory is not mounted here. No retained evidence is overwritten or
claimed independently verified. Earlier gate3543 evidence remains intact.

Worker fmt, locked all-target clippy with warnings denied and locked build
passed. No native e2e run occurred in the worker. The operator will run
the complete affected real G4 scenario once at the exact repaired candidate
before another full prepared host qualification.
