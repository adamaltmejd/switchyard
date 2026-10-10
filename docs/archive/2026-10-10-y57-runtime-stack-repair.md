# Y-57 runtime stack repair

The operator reported real candidate daemon aborts on 2026-10-10 at
c02f2677ce26e43d1cab1b4004eaae66f46a546d. The isolated full52 host run was
in /home/yard/validation/switchyard-y57-20261010, session58657, with log
/tmp/switchyard-y57-first-repair-host.log. Tokio workers reported stack
overflow and fatal runtime error. More than 50 scenarios visibly failed
before useful guarantee observations. The report said the legacy upgrade
scenario was still live. This audit claims no terminal total, passing
timeout or flaky retry. The live validation source and processes were not
touched.

## Diagnosis

The failure is explained by cumulative generated debug polling frames,
not a recursive sync/register/recovery call. Git::run_bytes embeds joined
stdout and stderr readers, each with an 8192-byte inline read buffer. That
state propagates into every enclosing Git, configuration and job future.
The added publication and recovery awaits increased the number of large
temporary future values in the unoptimized polling frames.

Measurements used rustc 1.98.1 on Linux x86_64, the locked dev build, and
the unchanged runtime settings. RUSTC_BOOTSTRAP=1 cargo rustc -p yard
--bin yard --locked -- -Zprint-type-sizes reported layouts. A separate
ordinary cargo build -p yard --locked supplied the baseline disassembly.
objdump -Cd and -R supplied frame prologues and direct/GOT call targets.
These are compiler measurements, not a local host reproduction or a
passing startup observation.

| Measured value in bytes | c02f267 baseline | Repair |
|---|---:|---:|
| Git::run_bytes future | 17400 | 576 |
| sync future | 19352 | 2528 |
| sync poll frame | 732136 | 59176 |
| command poll frame | 379096 | 107528 |
| register poll frame | 114808 | 47512 |
| landing poll frame | 741720 | 392936 |
| decide_intent poll frame | 183608 | 49016 |

The baseline current-default sync publication path has 1,924,032 bytes
of application frames alone: handle, command, sync, import,
publish_default, read_default, load, load_target, read_config, show,
run, run_with and run_bytes. The HTTP service adds 177,280 bytes through
poll_msg, api::serve, its envelope body and the dispatch closure. Their
sum, 2,101,312 bytes, already exceeds the normal 2 MiB worker stack,
before Tokio, generic polling adapters and deeper reader calls. If default
selection changed, refresh_default adds another 37,080 bytes. The source
and resolved calls establish this finite nested path; the overflow text
alone was not used to infer recursion.

After the repair, the largest measured application direct-call path from
handle is 742,816 bytes, through init/register/recovery. The corresponding
measured paths from landing and the execution task are 709,464 and
982,848 bytes. These sums cover application direct/GOT calls only; they
are not runtime stack high-water measurements or proof of all platforms.
The full unchanged host suite remains the runtime gate.

Raw layout output, baseline and repaired binaries, disassembly, relocations
and the frame extraction script are in /yard/proof/y57-*. They are candidate
proof artifacts and are not merged. No diagnostic flag or instrumentation
is added to the binary's build or runtime contract.

## Repair and controls

One Box::pin at Git::run_bytes allocates its existing bounded I/O future
on the heap. Its joined readers, child wait and output-cap checks are
unchanged. The outer future still owns the child. Timeout and cancellation
drop the pinned readers and then the child as before; inherited execution
locks still prevent uncertain children from being killed or losing their
reservation. No separate task, runtime setting, dependency or lifecycle
is introduced.

Per-target eligibility, approval ordering, target locks, persisted intent
refs, target CAS, shared project capacity, ancestry proof, default input
invalidation and unknown-owner publication refusal are unchanged. The
repair changes no assertion or scenario; all52 remain the operator's gate.
No operational .yard configuration, daemon or reviewer was changed, and
no development binary opened construction state.

## Validation

cargo fmt --check and cargo clippy --all-targets --locked -- -D warnings
pass. cargo build -p yard --locked passes. Host e2e was not run here, as
instructed. Fresh exact-head review and the full unchanged host suite on
the repaired head are required before approval.
