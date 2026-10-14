# Y-152 fixture dependency builds

The G8 fixture's three serial daemon builds now use one dependency target.
Each build first cleans the Yard package. This invalidates compiled source
and asset seals despite archived timestamps. The executable is then copied
to its existing distinct path before the next build. Separate source trees,
559f730, current HEAD with the comment-byte variant, and b9e374 are retained.
There is no concurrent builder or result reuse.

## Test-audit Gate

Disposition: keep every behavioral assertion and scenario unchanged.
G8's row covers the real six-field admission, different compiled package
with effort metadata, and exact-package fallback without metadata. The
existing boxed observations, native publications and historical-row checks
remain the observers. Expected runtime bytes still come from git archives
and the fixture's comment, not from Yard's output.

The new clean-command success check is setup validation, not a behavioral
assertion block. It prevents testing an executable with the wrong asset
seal. Removing package invalidation can reuse archived timestamps and
compile the wrong variant. The existing different-package control exercises
that failure; no duplicate scenario is added. Its existing sabotage comments
and hardest-case upgrade and adoption sequence remain unchanged.

No operator decision to weaken a guarantee is needed. No guarantee clause,
scenario or assertion block is removed. Test lines removed: one target
argument replaced. Binary lines removed: zero. Production code, spec and
dependencies are unchanged. The three separate dependency-build paths are
replaced by one target; the distinct executable paths remain.

## Qualification

An isolated boxed compile experiment used the same serial archive, clean,
build and copy sequence. All three executable SHA256s were distinct. Each
executable contained its exact source launcher, runner and package-lock
bytes. Only the current-source variant contained the fixture comment.
The current source was f84a3279b34470c3799e261803aed271b4e73242.
The candidate proof keeps the source archive, asset and executable digests,
exact sealed asset bytes, scripts and compile log in `/yard/proof/y152`.
Build intermediates and executables were removed after identity verification.

| Variant | Setup seconds | Clean/build/copy seconds | Executable SHA256 prefix |
| --- | ---: | ---: | --- |
| 559f730 pre-effort | 0.035 | 23.576 | 7efe6f47c2db38e7 |
| Current comment-byte metadata | 0.043 | 11.196 | 1b3609ef5fa6a390 |
| b9e374 fallback | 0.043 | 11.108 | fcddd9e1f53bc0c8 |

Total build/copy time was 45.880 seconds. Shared-target teardown took
0.071 seconds; retained source and executable teardown took 0.023 seconds.
Setup covers archive extraction and suffix writing. Build covers package
cleaning, compilation and executable copying. These are wall times from
the external proof script, not timing assertions. Runtime was not run.
There is no separate-build baseline for this box, so these numbers establish
identity and phase cost, not a reproduced saving. The first attempted
proof script could not run because Python was absent; its Perl replacement
completed successfully. Both scripts are retained in the candidate proof.

Boxed `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `cargo build --locked` passed. No e2e test was run: the worker instruction
reserves those runs for the host after approval.

The cited `/home/yard/validation/coordinator-20261014` evidence is absent
in this box. The ticket reports 93.682 seconds for separate builds and
55.103 seconds for shared dependencies with package invalidation, saving
38.579 seconds. These are supplied measurements, not reproduced host results.
The ticket's 753–764 second suites imply about a 5% saving if compilation
is on the critical path. No whole-suite effect has been measured here and
there is no claim to meet five minutes.

The operator must run the unchanged actual G8 scenario after the boxed
checks, then the full host landing gate. Preserve prior failures and
qualification manifests. Record host setup, build, runtime and teardown
timings in a new dated archive. Runtime and host teardown remain unmeasured
here; no binary barriers or timing assertions were added.
