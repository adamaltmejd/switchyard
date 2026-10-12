# Y-102 operator validation

Actual candidate and merged head: b4b0d12098d9fb8046795f728b6db93b15a2f5e2.
Base: 7a4c95a582f8db79224ca321ef4a33cb6e78f0e7. Attempt 95,
approval 116, landing 2053. The landing intent is resolved, the attempt
ended landed and the ticket is done. Cleanup 2059 ended removed.

Candidate fmt 2045, clippy 2046 and build 2047 passed. Correctness review
2048/check1078 passed with no findings. Fresh head, proof and public diff
matched the operator audit before approval. Actual merged fmt2054,
clippy2055, build2056 and host e2e2057 passed.

The full host gate ran the unchanged configured concurrency and deadline:

```sh
PATH=/home/yard/validation/podman-shifting-profile/bin:$PATH \
YARD_E2E_DURABLE_INPUTS=/home/yard/validation/switchyard-durable-inputs \
cargo test -p e2e --locked
```

Result: 75 passed, zero failed, 708.77 seconds. Both packaged audit
scenarios passed, including the selected native controller cancellation,
real host ownership refusal after publication, retained capacity and
ineligible report, then successful checked release with the same bound
report and continuation. Source and changed blocks were read against
the full original ticket and test-audit before approval.

Failed host2028 remains failed: 74 passed, one failed, 1013.19 seconds.
Its history observer missed an already consumed execution start. Review2037
then found a wrong expected report nesting before the positive control.
The repaired candidate uses existing history lookup and compares result
output with the prior full bound publication, while the fixture payload
is independently asserted. Review2011 global-box observer failure is
also retained. No retry loop, deadline/concurrency masking, assertion
weakening or production test barrier was added.

Retained operator evidence: /tmp/switchyard-y102-host2057-terminal.json,
/tmp/switchyard-y102-host2057.log, /tmp/switchyard-y102-host2028-terminal.json,
/tmp/switchyard-y102-host2028-failed.log and
/tmp/switchyard-y102-review2037-terminal.json. Candidate audit is under
/home/yard/validation/y102-b4-review-20261012/. Read-only runtime observations
and exact host test PID/start identity are under
/home/yard/validation/cache-race-observation-20261012/. Sampling does not
prove a runtime lock holder or historical cache-failure cause.

This is isolated FUSE-profile Linux x86_64 evidence. It is not native-store,
Linux arm64, Mac or operator sabotage evidence. The measured run exceeds
the historical five-minute budget; no scenario or budget was changed.
The operational daemon, binary, reviewer and active native store were
unchanged. No push, publication or installation occurred.

Y-100 and Y-93 retain their full original integration requirements.
Y-105's separate failed host2042 remains failed; its later landing2058
conflicted in ARCHITECTURE.md before any new host gate. Neither approval
nor this record completes Y-105, its parents or the release scope.
