# Y-100 operator integration validation

Candidate and actual merged head: 7272212aa161b7ab814b28529914ac327d0cecc9.
Base/old target: 1c72d012d18d2040b2bd121a35d07553f622a787. Attempt96,
approval118, landing2070. Intent resolved, attempt ended landed, ticket done.
Cleanup2075 ended removed.

The full original revision2 contract and sizing addendum were read, along
with the complete candidate diff and integration audit. Named Y99/Y101/Y102
actual landed heads are ancestors of the combined base. The candidate
corrects the stale future-selection sentence and documents noninherited
explicit profile selection. Source and tests otherwise remain unchanged.
The operator checked source selection/admission, canonical trusted blobs,
empty broad agent environment, native boxed invocation, sole eligibility
and checked ownership against the complete original acceptance mechanisms.

Candidate fmt2066/clippy2067/build2068 passed; correctness2069/check1086
passed with no findings. Fresh head/proof/public diff matched the audited
candidate before approval. Actual merged fmt2071/clippy2072/build2073 passed.
Full host2074 passed 75 scenarios, zero failures, 576.03 seconds. Both
complete packaged audit scenarios and shared publication controls passed,
including selected cancellation, real ownership refusal, retained capacity
and report ineligibility followed by successful checked release.

The host command retained the configured deadline and concurrency:

```sh
PATH=/home/yard/validation/podman-shifting-profile/bin:$PATH \
YARD_E2E_DURABLE_INPUTS=/home/yard/validation/switchyard-durable-inputs \
cargo test -p e2e --locked
```

Evidence: /tmp/switchyard-y100-host2074-terminal.json,
/tmp/switchyard-y100-host2074.log and
/home/yard/validation/y100-7272212-review-20261012/operator-integration-audit.json.
Host test identity and shared read-only observations are under
/home/yard/validation/cache-race-observation-20261012/. Sampling is not
lock-holder or cache-cause proof. Previous child failures remain preserved.

This is isolated FUSE-profile Linux x86_64 validation, not native-store,
arm64, Mac or executed sabotage evidence. The run exceeds the historical
five-minute budget; no scenario or budget changed. No operational binary,
daemon, reviewer or active native store changed. Nothing was pushed,
published or installed.

Y93 still requires its complete original integration and fresh actual host
acceptance. This landing unblocks that work; it does not complete Y93, Y64
or the release. Y105's separate failed gate and merge conflict remain failed
evidence, with its latest candidate still awaiting review and actual landing.
