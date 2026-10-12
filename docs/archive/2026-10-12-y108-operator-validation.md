# Y-108 operator validation

Y-108 attempt 105 landed as cfc29ca253680ad1e4e7c7245a4b7d980415517b.
Its base was 712f88a3d0baa0ebb50afa0a746ff3b62405b011.
The checked and consumed tree is 6ab53551aa494ebcbcfa8bf9b4f8f817c6b860e4.
This report changes no runtime, configuration or test assertion.

## Evidence and disposition

The original candidate diff and both incremental repairs were read in full,
including the protected live-revision contract and G9 shown-by clause.
The final CLI diff matches committed Git diff with one extra CLI newline.
Git diff SHA256: 5348776d836853099534400acb28a4c3387adf907733f749f672ed39fc3489d9.
Private evidence: /home/yard/validation/y108-571a3f-review-20261012.

Candidate 571a3f839920e59e0f2d000d8222a2cdefab5b70 passed live review teardown
and held implementation/edit, then failed the unchanged read-only handoff.
The daemon panicked at plan.rs:355 on an unavailable unused workflow seat.
The direct host failure remains preserved; later passes do not erase it.
Candidate b701196ca98d7e91d71feddb6f26c8a69d930da3 replaced that index with
the existing unknown-seat error; selected workflow validation still applies.
Six focused host controls passed on that repair. The initial five-case batch
included head/proof/template identity; quiescent plan CAS passed separately.

A startup-only mutation replaced the first execution-bound attempt reader
in supervise::implement with the latest attempt reader. Both existing held
implementation/edit and read-only scenarios still passed on b701196.
Their model holds were after startup. This exposed a coverage gap.

The final candidate extends the existing G9 scenario rather than adding
a twin. A real worker holds the sole capacity slot; the admitted target
execution waits unreserved. A revision occurs before startup. After release,
its actual model request carries the original instructions; the successor
receives revised instructions and the labelled operator revision diff.
Existing ticket-edit, explicit-stop, candidate and gate assertions remain.
The unchanged scenario passed: 17.06 seconds, 18.987 seconds including build.
The precise startup-only mutation failed directly at G9 line 165, with
revised instructions instead of original ones: exit 101, 14.80 seconds.
No timeout passed. Exact source and test binary were restored; clone clean.

Boxed checks 1154/1155/1156 passed fmt, clippy and build.
DeepSeek review 2223/check 1157 passed in 792.657 seconds. It reported
94,266 output tokens, including 83,923 reasoning tokens, across 83 messages.
Those are reported counters; elapsed time includes tools and request latency.
No pure provider throughput claim follows.

The P2 unshown revision-refusal controls during landing intent or pending
repair and P3 superseded-gate raw historical result semantics are carried
in mandatory Y-111 revision 4. The operator read the guards, check suppression
and node/input qualification. Landing over nonblocking findings does not
prove those clauses or authorize dropping them. Y-111 precedes Y-109 and Y-80.

Approval 127 names the exact head and empty-proof SHA256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855.
Landing 2224 passed fmt 2225, clippy 2226, build 2227 and real host e2e 2228.
The full Linux x86_64 suite passed 78/0 in 667.26 seconds.
The host gate elapsed 682.304 seconds including its build.
Audit 7723 records the landing; 7724 marks the ticket done; 7725 ends the
attempt. Cleanup 2229 ended removed at 7727. The consumed head/tree match.
The historical five-minute budget is exceeded; Y-111 must judge observed
cost, without deleting required coverage solely for timing.

The real-Pi cache probe under /home/yard/validation/pi-cache-cfc29c-preaudit-20261012
supplied native provider cache counters to the existing usage scenario.
It passed, while dropping reads or writes failed the existing total assertion.
That temporary experiment is restored and carried into Y-111; no coverage
change has landed and final-head release validation remains required.

Full Y-62 integration, Y-109, Y-110, the early consolidation checkpoint,
remaining redesign, final cleanup, migration/restore and platform validation
remain. Y-81 stays parked. No release, push, installation, daemon replacement
or reviewer change occurred. The user's separate AGENTS.md edit is excluded
from this archive commit and its exact bytes are retained privately.
