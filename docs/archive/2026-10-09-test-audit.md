# Y-54 assertion audit

This is the test-audit gate for the Y-54 assertion change, not a whole-suite
release sweep. The new scenario is
`claude_api_credentials_stay_on_host_and_publication_requires_proof` in
`g08_review_is_a_publication.rs`. It uses the real daemon, Claude harness,
boxes and existing scripted host Anthropic fixture. No binary test seam,
sleep, retry, clock wait or public model request was added.

| Block | Disposition and clause | Sabotage | Observer and outside expectation |
|---|---|---|---|
| Conflicting authentication sync | Add, G6 named `provider_login_conflict` refusal, no write | Accept provider and login together | CLI names `agents.worker.provider_login_conflict` from the spec; canonical Git head and audit rows equal their pre-call snapshots |
| API publication with removed init proof | Add, G8 only the fresh proven seat counts | Remove `checks::current` registration predicate | Store has two review checks, first unregistered and second registered; approval agrees with the second check's identity; finding text comes from the fixture |
| Guest absence and host receipt | Add, G12 API key absent in guest, received by host | Pass literal key to guest env or remove injection | Real seat command searches process env and mounted/temporary files; zero matches expected from the threat model; host requests carry the fixture key, API version and selected model |
| Same-project subscription login | Add, G6/G8 corrected login succeeds | Break Claude login routing while adding API mode | Approval from CLI history and host `Authorization: Bearer` equal to the fixture token |
| Existing credential absence helper | Extract, G12 observations unchanged | Existing credential leakage sabotage | Same real guest probe and zero expected matches as before |

The API scenario removes the first seat's actual stdout init frame in a
host pinfold wrapper, preserving the child exit code. It does not alter
Claude's MCP connection or its publication. This is the harder case: a
published pass with no durable execution proof, rather than an exit with no
publication. The subsequent proven API seat is its positive control. The
authentication conflict is followed by successful API and login work in
the same project.

The existing Pi no-proof block observes stderr registration, while Claude
proves registration on stdout. The existing Claude containment block uses
subscription authentication. Neither exercises the direct API-key route
with Claude's registration frame. They remain. The guest search helper is
shared rather than copied; all prior callers keep their assertions.
No guarantee or existing clause was weakened.

The operator's list is empty. All four gate questions are answered above.
The diff removes 15 test lines and 11 binary lines; removals are relocation
of the shared probe and replacement of login-only selection logic, not
deleted acceptance coverage.

Fmt and all-targets locked clippy passed. Runtime and sabotage runs were
not performed: this ticket explicitly reserves host e2e for the landing
gate. No runtime pass is claimed.
