# Construction scope and fresh-base restart

This records operator decisions after Y-92 actually landed. It does not
complete Y-93, Y-95 or the release.

## Y-95 retained clone

The old attempt 85 retained candidate 17b36ff2f8e909800e4e89478799560438a4bfc6
on base e4ce7d5. Its revision 5 worker could not obtain canonical 52ad811: the
object was absent, the clone had no remote, and network fetch returned 403.
It reported that boundary and produced no new commit. This was a construction
source-delivery limitation, separate from container listing contention.

The operator parked Y-95 before abandoning attempt 85. The ticket retains
its entire original acceptance contract. Revision 6 carries the reviewed public
proof/source patch and three immutable historical archives through the normal
public body, plus the complete architecture union. Both patches were checked
against the clean new base. No private worker input, clone or store was edited.

Fresh attempt 87 started from canonical 52ad811 and produced
347d86440be0ed8ed145a649a2fa9727b2c14ef3. Its admission/G2 sources and three
historical archives are byte-identical to reviewed 17b. Its architecture
retains all Y-92 clauses and all four required-proof additions. The worker
corrected one truncated word in the supplied draft. Its new dated integration
audit maps the complete parent contract and distinguishes inherited evidence
from fresh unrun host validation. Approval and actual landing remain required.

## Y-93 partition

Candidate edf51f3a51a181c3d45029cf647c3b80b8cbe7f1 changed only 77 lines in a
dated sizing/test-audit assessment. It implemented no runtime or spec change.
The two proposed implementation boundaries were read against the original
G8/G12/G13 scope. The parent was parked before attempt 86 was abandoned.
Retirement resolved proposals 216/217; the operator filed their required work
as new parked tickets, preserving admission and dependency evidence.

- Y-99: audit filesystem isolation, frozen source/proof and execution/node
  writable state; no predecessor mount or automatic resume. Depends on the
  actually done Y-91/Y-92.
- Y-100: explicit packaged runner selection, deterministic trusted invocation,
  selected trusted inputs, native registration/route, truthful eligibility and
  cancellation/teardown. Depends on Y-99 and stays parked until it lands.
- Y-93: complete original integration. It retains Y-91/Y-92 dependencies and
  now also depends on Y-100. Revision 2 retains the whole original body and
  requires proof of every original clause after both children actually land.

Y-99 was read back parked, with resolved dependencies, before unpark.
Y-100 and Y-93 remain parked. The active dependency graph resolves without a
cycle, with completed ancestors treated as terminal. No sizing document or
child filing marks a runtime guarantee done. Y-64 and downstream release scope
continue to depend on Y-93. Y-81 remains parked beyond this release.

These tickets reuse the execution lifecycle, slot, registration and checked
teardown. No independent owner/recovery path, unmeasured speedup or Pinfold
feature is admitted. All host e2e remains exclusive, real and operator-owned.
The configured implementer/reviewer, operational binary and daemon are
unchanged. The temporary FUSE/backup agreement remains in force.
