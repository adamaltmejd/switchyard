# Y137 implementation partition

On 2026-10-13, Y137 attempt 134 produced head
`7d441e219791261716aa68b00b124ead71b40010` against
`ae326097c4d5ac3fc512c0b048d5e40af722a069`. Its only change was a
scope assessment. Checks 1341 through 1344 passed, but this was not an
implementation of the ticket. The operator parked Y137 and abandoned the
attempt without landing it. Its candidate and review evidence were retained.

The source trace identifies three changes with different runtime boundaries:

- Y139 derives implementation settings from each execution's recorded
  revision and includes changed definitions in the existing handoff. It
  depends on landed Y135.
- Y140 adopts explicit committed definition, staging and image sources for
  ordinary plans under revision CAS. It depends on Y134, Y135 and Y139.
- Y141 extends that adoption to the running single-node packaged controller.
  It depends on Y139 and Y140.

All three tickets were created parked. Proposals 365, 366 and 367 were
rejected as duplicates of these operator-created tickets. Y137 revision 2
retains its complete original body and depends on all three actual landings.
The shared prerequisite is the next dependency-ready implementation.

`store/attempts.rs::for_execution` selects recorded admission evidence but
leaves the original attempt's implementer settings. Implementation startup
and launch read those settings. The handoff currently compares plan JSON.
The source-adoption and packaged-runtime changes therefore remain separate
from this shared settings prerequisite. Check 1344's P3 trace correction is
recorded here: the actual symbol is
`store/executions.rs::consumed_definition`, not `consumer_definition`.

The partition does not complete Y137, Y136, Y109 or Y62. Their full ordinary
and packaged acceptance remains required. Y138 still follows Y137 and covers
stopped packaged ownership using the released execution's frozen evidence.
Y63 and Y65 remain attached to Y62; Y81 remains deferred and parked.

Y135's combined staging/image foundation case does not prove independent
source qualification. The dependent adoption work must hold the exact image
and context fixed while separately changing gate definitions, declared agent
settings and selected staged bytes. Real host consumers, immutable rows and
checked ownership remain the required observations. No assertions or controls
were removed by this partition.

This commit records an operator decision only. It changes no runtime or test
code and needs no additional native suite. The dependency refresh's prior
local and Linux CI runs remain evidence for their exact tested sources.
The installed daemon, binary and reviewer are unchanged.
