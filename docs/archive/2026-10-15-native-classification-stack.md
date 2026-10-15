# Y73 native landing stack measurements

The operator observed a fatal Tokio worker stack overflow in the existing
grouped-finding host case at 530fe30. The retained store showed completed
classifier 35, landed execution 36, running cleanup 37, running landing 38 and
running landing gate 39. The host captured no overflowing stack frames. Those
rows locate the work in progress; they do not establish the failing call chain.
Neither that case nor a whole native suite is claimed to pass here.

## Measurements before choosing the fix

Local measurements use Linux x86_64, rustc 1.99.0
`b940084d7 2026-09-28`, the locked dependencies and the normal unoptimized
development profile. No daemon or Yard CLI was run. Compiler diagnostics and
static disassembly require no runtime, project state or model access.

For each source version, compile the real binary with:

```sh
RUSTC_BOOTSTRAP=1 cargo rustc -p yard --bin yard --locked -- -Zprint-type-sizes
objdump -d -C target/debug/yard
```

The compiler switch prints layouts only; no diagnostic path is added to the
binary. Historical sources were extracted outside the workspace, excluding
`.yard`. Recompilation was forced when reusing the target directory. The
prologues below are the stack-probe decrement plus the final stack decrement
and any callee-saved register pushes, excluding the return address.

| Source | Landing future | Landing poll frame | Child-dispatch future | Child-dispatch poll frame |
|---|---:|---:|---:|---:|
| pre-Y73, 0e49425 | 70,744 | 762,696 | no `run_child` wrapper | no wrapper |
| retained failure, 530fe30 | 73,712 | 1,208,152 | 64,616 | 152,184 |
| previous repair, f8a7254 | 73,712 | 1,208,152 | 64,616 | 152,184 |
| this repair | 73,712 | 949,912 | 56 | 87,688 |

All numbers are bytes. The native gate's capacity future is 64,568 bytes in
both the pre-Y73 and retained sources. The new wrapper embeds it by value and
is itself embedded at both landing call sites. Thus the roughly 3 KiB growth
of the landing future hides 445,456 bytes of additional landing poll-frame
reservation. The execution-dispatch poll frame also grew from 349,048 to
361,320 bytes. These are concrete stack-pressure measurements, not a captured
host backtrace or a calculation of the maximum dynamic stack depth.

An initial experiment heap-pinned only the classifier call in `land`. That
reduced the landing frame to 1,139,896 bytes. It was discarded: it left most
of the measured native child-dispatch pressure in place, including the wrapper
on the gate path active in the retained store snapshot.

## Selected repair

Heap-pin only the measured capacity future in the existing `run_child`
dispatch, before awaiting it. Both initial and retried native gates use that
same dispatch. The change reduces the combined reservations of the landing
and child-dispatch poll frames by 322,736 bytes. The landing state-machine
size stays unchanged because the classification branch still determines its
largest suspended variant. No broad boxing, new owner or new execution is
introduced. Reservation, target locking, gate execution, error recording and
checked release stay in their existing order.

The retained and repaired landing prologues reserve respectively
`0x126000 + 0xf28 + 48` and `0xe7000 + 0xe68 + 48` bytes. The child-dispatch
prologues reserve respectively `0x25000 + 0x278` and `0x15000 + 0x688` bytes.
Compact compiler and disassembly excerpts are retained under
`/yard/proof/y73-stack` for this candidate. Runtime thread stack settings,
deadlines, concurrency, ownership controls and every test assertion are
unchanged. The earlier directory-mount, validation-boundary and retry-recovery
repairs remain in place.

Candidate formatting, Clippy and build checks run again for this repair.
All real host e2e, including the unchanged grouped-source, required-check and
landing assertions, remain the operator's gate. Host confirmation of the
crash repair is pending.
