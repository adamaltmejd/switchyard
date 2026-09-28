# pinfold's daily pass in every test

This follows `2026-09-28-pinfold-0.0.8.md`. On 0.0.8, `box list` was the
largest pinfold cost in the suite: 286 s over 68 calls, with a median of
4.2 s.

## Cause

A podman wrapper on the test PATH logged every podman call pinfold made,
along with the pinfold command that made it.

- A `box list` is one `podman ps --all`, which takes about 0.05 s.
- pinfold runs its daily maintenance pass before its first working
  command. It stamps the run in `$XDG_STATE_HOME/pinfold`.
- Each test machine set its own `XDG_STATE_HOME`, for Yard's state. The
  daemon passes that to pinfold, so every test started with no stamp, and
  its first pinfold command ran the pass. That command was the `box list`
  in `yard init`'s reconcile.
- All 55 tests ran it. Each pass made a median of 107 podman calls and took
  9.7 s, 577 s in total.
- The pass is slow because it lists boxes and images once per image
  source, and there are 64 caller names on this host (pinfold #69).

## Change

The harness gives pinfold one state dir, `/tmp/yard-e2e-state`, shared by
every test, as a host has one. Tests already shared one pinfold cache.
Every pinfold wrapper on the daemon's `PATH` sets it, and so does the
harness's own pinfold call.

## Timing

| | per-test state | shared state |
|---|---|---|
| maintenance passes | 55 | 1 (133 podman calls) |
| `box list`, 68 calls | 286 s (median 4.2 s) | 20 s (median 0.07 s) |
| wall | 198 s | **134 s**, 55 of 55 |

The 198 s run had no podman wrapper. The 134 s run had one, and it
still ran faster.

A run whose stamp is older than a day still pays one pass, in its first
test.
