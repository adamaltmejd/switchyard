# Y-93 operator validation

Y-93's complete revision 2 contract was reviewed before approval. The
candidate integrates the landed Y-91, Y-92, Y-99 and Y-100 implementations.
The original Y-64 integration and downstream release scope remain required.
Checkpoint resume remains Y-65 work.

## Identity and decision

- Candidate and actual landed head:
  `869797aaba92b5da65d4e27bd4bd6a9f58160c19`.
- Base and previous target:
  `c6daac55483db9ba8a25e10a6c8a9b45b9ab737e`.
- Attempt 97, approval 119, landing execution 2082.
- Published review 2081/check 1091 passed without findings.
- The entire diff and integration assessment were read. The change updates
  the general harness-state contract and records the complete integration
  assessment. Production code and assertions are unchanged.

## Actual landing evidence

Merged-head fmt 2083, clippy 2084 and build 2087 passed. Real host e2e
execution 2089 passed all 75 tests, with no failures or ignored scenarios,
in 924.47 seconds. Both packaged-audit scenarios passed: missing assets
followed by the corrected image, and bounded iterations with native
publication. The latter includes cancellation and checked ownership
release. Existing publication, route, containment and shared-capacity
scenarios also passed in this combined tree.

The landing intent resolved with the exact previous and merged heads.
The attempt ended landed, the ticket is done, and cleanup 2091 ended
removed. Operator sync consumed the landed head. These are actual queue
results, not a worker's claim or an inherited child-only pass.

The configured host gate used the existing isolated validation profile:

```sh
PATH=/home/yard/validation/podman-shifting-profile/bin:$PATH \
YARD_E2E_DURABLE_INPUTS=/home/yard/validation/switchyard-durable-inputs \
cargo test -p e2e --locked
```

The gate retained default concurrency, assertions and deadlines. Its
runtime profile uses FUSE; this pass is not native-overlay evidence.
The operational binary and daemon were unchanged.

## Limits and retained evidence

The terminal attempt output and complete gate log are retained at
`/tmp/switchyard-y93-host2089-terminal.json` and
`/tmp/switchyard-y93-host2089.log`. Private source-audit evidence remains in
`/home/yard/validation/y93-869797a-review-20261012`.
Read-only runtime observations cover only part of this gate and do not
establish a cause for earlier cache or native-listing failures. Earlier
failed results remain failed. No sabotage, arm64, Mac, final migration or
release completion is inferred. Y-64 still needs its full integration
acceptance. No push, publication, installation or switchover occurred.
