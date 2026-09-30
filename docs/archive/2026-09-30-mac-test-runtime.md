# Mac test concurrency, 2026-09-30

Follow-up to the lean suite audit. Source baseline: `d1372d9`. All 47
scenarios, assertions and production behavior stayed unchanged. The
existing test runner's concurrency setting was the measured variable.

Host: macOS 27.0 build 26A428, arm64, 10 CPU cores, 24 GiB RAM,
Apple container 1.5.0 and pinfold 0.0.9. Runtime use was coordinated with
the pinfold chat under the operator's prior authorization.

| Run | Threads | Result | Elapsed |
|---|---:|---|---:|
| Cleanup final, before storage recovery | 2 | 47 passed | 257.05 s |
| First concurrency experiment | 4 | Interrupted after disk exhaustion | Invalid timing |
| After storage recovery | 4 | 47 passed | 188.25 s |
| Immediate control after that run | 2 | 47 passed | 230.97 s |

The recovered comparison saves 42.72 seconds, 18.5%. Both successful
runs had zero failed, ignored or filtered tests. Four threads ran first
with a fresh builder; two threads ran immediately afterwards. The profile
image and harness caches were retained. This is one paired measurement;
host load and cache state still affect elapsed time.

Recommend four scenario threads on the operator's Mac:

    cargo test -p e2e --locked -- --test-threads=4

The first experiment failed during OCI export with `no space left on
device`; later commands also saw a read-only builder filesystem or host
ENOSPC. Host free space was about 1.3 GiB. It was stopped rather than
waiting for held fixtures to time out. Only its own child processes,
temporary fixture directories and recorded image names were retired.
No test assertion was removed or weakened in response to this failure.

The idle builder cache occupied 16 GiB by du. After confirming that only
the builder remained and no worker box was running, the builder was
stopped and deleted with the runtime's supported commands. Host free
space immediately rose to about 59 GiB and later to 84 GiB. Recovery is a
confound in comparisons with earlier runs; the concurrency result above
uses the recovered two-thread control instead.

The harness already builds Yard once and shares pinfold state and harness
caches. Image-name cleanup is already deduplicated. Repeated prune/image
removal and project Git setup remain profiling candidates, without a
measured reason to change them. No new helper, dependency or binary knob
was added. Linux concurrency was not measured. Concurrent pinfold cleanup
still deletes the shared builder; this runner setting does not solve that
separate cross-suite coordination problem.

Evidence in /private/tmp: yard-mac-four-threads-2026-09-30.log,
yard-mac-four-threads-recovered-2026-09-30.log and
yard-mac-two-threads-control-2026-09-30.log.
