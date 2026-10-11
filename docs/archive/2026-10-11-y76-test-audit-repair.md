# Y-76 source-gate assertion repair

The operator rejected candidate `4f55db7` before a host run. Its API and
login entrypoints dispatched the same four phases and assertion blocks.
The previous audit named authentication paths, but no different accounting
mechanism. That did not meet the fixed-list rule or Gate question 3.
No runtime failure or pinned route-specific usage difference was observed.
The previous archive remains a record of that rejected design.

## Dispositions and Gate answers

Keep one G7 scenario, `claude_usage_is_optional_and_execution_local`, on
the real API route. Delete the login twin and the boolean dispatcher,
including their authentication/configuration branches. There is no loop
that reruns the accounting blocks under another credential selection.
The shared `check`, `execution` and `raw_cost` helpers still serve one test.
All comparison expressions and the four accounting phases are retained.

Each retained block serves G7's shared API/login reader clause:

- Unknown-cost completion and later explicit-zero completion distinguish
  absence from a numeric observation. Defaulting absent cost to zero or
  discarding explicit zero breaks them. Expected values are the host's
  measurement-field transformations and the spec's metadata contract.
- The failed resumed result compares nonzero input/cache/output counters,
  result replacement and partial completeness. Adding terminal totals to
  streamed observations, including old session usage, or marking failure
  complete breaks it. Expected counters come from the scripted host
  responses, coverage from G7, and outcome from the host's HTTP 400.
- The interrupted resumed turn has multiple content blocks and a held next
  reply. Terminal-only persistence or counting repeated message ids breaks
  its partial-token/null-cost block. Expected increments come from fixture
  replies; transcript EOF must contain no result. Ending the real box drains
  stdout. CLI identity and history distinguish failure from a timeout.
- Reopening preserves the interrupted execution while the fixture remains
  held. Dropping partial observations during reopen breaks this equality.
  The external pre-reopen record supplies its expected value.
- Final resumed success compares only new fixture counters, full metadata
  and explicit zero. Including earlier session totals or downgrading Claude
  resume completeness breaks it. Each resume identity is the preceding
  fixture execution, not an identity derived from Yard's accounting logic.
- Raw failed/resumed estimates are compared with the fresh raw harness
  estimate for the same nonzero input/output/cache counters and fixture reply
  counts. Accumulating session cost again breaks this block. The baseline
  is actual unmodified harness stdout, not a guessed bill or catalog rate.

Observers remain CLI JSON/history, host fixture counters, actual harness
transcripts and raw stdout. The hardest case is repeated content blocks
across failure, interruption, reopening and further resume. Fresh Claude,
Pi and Codex scenarios cannot expose this combined shared-reader mechanism.
The removed login run supplied identical observations and sabotages, so it
had no independent Gate question 3 answer. One copy of its executed blocks
is removed; none of the surviving assertion expressions is removed.

The independent route evidence remains in G8's
`claude_api_credentials_stay_on_host_and_publication_requires_proof`:
host requests observe injected API credentials and protocol headers, then
successful subscription login with bearer authentication in the same project.
Its sabotages concern credentials and registration, not usage arithmetic.
The login containment scenario also continues to exercise real login work.
Those existing blocks are unchanged. Neither is claimed to establish an
observed route-specific accounting difference. No such difference is known
from a host run of this candidate, so no new route-specific usage block is
invented. Both routes still select the same production reader.

The full API/login optional-estimate, cache, failure, interruption and resume
contract in Executions is unchanged. G7's Shown by text now names the one
accounting scenario and the independent G8 authentication controls. No
promised behavior or required accounting observation was removed. The
operator's requested twin removal is resolved; no further approval item
is inferred. Test source lines removed: 62; added: 42; net removal: 20
(including rustfmt reindentation). Binary lines removed: zero.

## Validation

`cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`
pass. No host e2e or sabotage run was performed in the worker box. No daemon
was started and no construction configuration was edited. Exact-head host
validation, fresh review and landing remain operator steps; this source
repair does not claim runtime acceptance.
