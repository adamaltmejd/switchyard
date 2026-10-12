# Y-105 operator validation

The complete Y-105 revision 4 contract and candidate diff were reviewed.
The source audit preserved exact failed evidence, original prepared input,
checked ownership, active approval, remaining selected gates and final CAS.
Y-104/Y-96/Y-61 full integration and Y-106 omission decisions remain required.

Candidate: `d714687c8a1bc298a0de9321740b4417a8de3ede`.
Actual merged head: `b9429b066e6acb1a0e442ec70c0cfff568f25a4c`.
Previous target: `389bb67ad779f51a4a71051fb94e8bee0c71a5fa`.
Attempt 94, approval 120, landing 2093. Review 2090/check1095 passed
without findings. Fresh head/proof and the decoded complete audited diff
matched before approval. The automatic union tree was independently
verified in an operator-owned repository; no canonical objects were written.

## Runtime evidence

The isolated unchanged G3 ownership scenario passed: one scenario, 32.17s.
Removing only admit's answer-time predecessor proof made that scenario fail
at g03_landing_cas.rs:521: the answer incorrectly returned accepted:true
instead of stale. It failed in 14.92s, not by timeout or build error.
The exact original source bytes were restored and SHA256 verified:
`182fb696b19ebb1adaa7ec046d4769486597c7ac7de06eed7168732443bbadb6`.
This experiment used only the operator-owned reconstructed checkout and
ran between queue host gates. It changed no worker clone or operational
binary. Its retained candidate diff from the base is intentional.

Actual merged fmt2094, clippy2095 and build2096 passed. Host e2e2097 passed
75 tests, zero failed/ignored/filtered, in 1149.08s. The landing intent
resolved with the exact previous and merged heads, the attempt ended landed,
the ticket is done, and cleanup2103 ended removed. Operator sync consumed
that merged head without changing the target while Y-64's landing ran.

The host gate and focused experiments used the existing isolated FUSE
validation profile and durable public input directory with unchanged tests,
default suite concurrency and deadlines. This is Linux x86_64 evidence,
not native-overlay, arm64 or Mac release evidence. Y-64's approved queue
landing began automatically. This record was retained outside the tracked
repository until Y-64 landed, then imported between landings.

## Limits and retained evidence

Terminal Yard output and retained gate detail are
`/tmp/switchyard-y105-host2097-terminal.json` and
`/tmp/switchyard-y105-host2097.log`. Focused control/sabotage logs, restored
source identity and exact union proof are in
`/home/yard/validation/y105-d714687-review-20261012`.
Earlier host1969/2042 failures and conflict2058 remain failed evidence.
Separate repair-path runtime coverage and diagnostic findings remain
carry-ins for the full parent integration/final audit; no closure is inferred.
Read-only process observations do not establish earlier cache-failure causes.
No push, publication, install, reviewer switch or daemon replacement occurred.
