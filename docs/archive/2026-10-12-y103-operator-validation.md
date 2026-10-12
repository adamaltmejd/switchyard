# Y-103 operator validation

Y-103 landed as `4f78ea237c721a904b4696784e36615a07b67af3`, from
`a121749d5e799fac5ac4fee1ca37e274df7e9f93`. Attempt92, approval108 and
landing1934 identify this exact candidate. Its canonical mutation intent is
resolved. Cleanup1943 ended as removed. The operator consumed the landing
with `yard sync`; this is an actual implementation landing, not sizing.

The operator read the complete candidate against the ticket, including the
prepared-input identity, predecessor ownership checks, retained child
qualification, target serialization, final intent and CAS. Review1017
passed with no findings. Candidate fmt1013, clippy1014 and build1015 passed.
Actual landing fmt1935, clippy1936 and build1937 passed.

Host gate1938 passed all 75 scenarios, with no failures or filtered tests,
in 686.43 seconds on yard-sthlm, Linux x86_64. Its configured command was:

```sh
PATH=/home/yard/validation/podman-shifting-profile/bin:$PATH YARD_E2E_DURABLE_INPUTS=/home/yard/validation/switchyard-durable-inputs cargo test -p e2e --locked
```

This uses the existing isolated FUSE profile. It proves neither native-store
performance nor Linux arm64 or Mac release validation. The measured duration
exceeds the historical five-minute budget; no scenario or assertion was
removed to meet that budget. No runtime, deadline or concurrency setting
changed for this landing.

The continuation scenarios retain the exact prepared Git input and passing
children through the existing execution lifecycle. Their real held child
and external Git/store observers cover restart and changed target input.
The G8 infrastructure-error retry retains its passing child and truthful
failed evidence; its broad no-check-publications assertion remains intact.
The invalid selected-child guard repair was read in source. This host pass
does not establish direct execution of both narrow invalidation branches.
No independent sabotage run is claimed.

The earlier host gate1906 remains failed: 74 passed and one failed in
466.55 seconds. Its G7 closed event list omitted the new prepared event.
That failure and review1008's two integrity findings are preserved. The
candidate repaired the closed event list, restored the broad G8 assertion
and retires invalid continuation evidence transactionally before intent.
Its new review1008 repair archive corrects the earlier audit claim without
rewriting that historical archive.

Y-104 remains required for exact failed/skipped merged-evidence acceptance.
Y-96 and Y-61 retain their full integration contracts. This landing does
not complete those parents or the release through Y-80. Y-81 stays parked.
The frozen operational daemon, implementer and reviewer remain unchanged.
No push, publication, binary installation or service switch occurred.
