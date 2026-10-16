# Y80 physical fixture paths on Mac

2026-10-16. Continues `cc54164ef7a68e06dc6d9572a083225df29b53c0`.
The operator read that full product diff and review 2083 with no finding.
Its legacy alias ownership repair remains unchanged.

Operator diagnostic CI `38925951241` ran one real 0.0.3 G15 case on exact
`b670ca0840c1891819696d21eb3712a1ffd8cd90` after required default profile
preparation. Old box startup succeeded, with six scripted requests and
empty container stderr. The case then failed at
`g15_plans_and_proposals.rs:313`: the old registry contained
`/private/tmp/ye-7060-g15-upgrade/p`, while the fixture expected
`/tmp/ye-7060-g15-upgrade/p`. These are supplied operator observations;
raw evidence remains with the operator. The focused startup success does
not explain or erase the original full Mac gate's legacy startup exit 1.

The real old CLI correctly records its physical canonical working directory.
Machine setup now canonicalizes the existing `/tmp` base through the host
filesystem before appending the short machine name. Project, state, backup,
copy and separate restore paths already derive from that machine root;
they now share the physical path spelling. No old source or registry
expectation is changed, and no Yard result is used as the path oracle.
The existing G7 explicit symlink init/sync/landing representative is
unchanged, preserving the independent product-alias regression case.

Test-audit disposition: retain all G5/G6/G15 migration/restore assertions
and the existing hard-case machines. The material risk remains changed
ownership, decisions or recoverable effects across upgrade and old restore.
The fix corrects host setup, not the expected old registry value. Host
filesystem canonicalization is the external identity oracle, while the
old CLI independently writes the registry being checked. Existing sabotage
comments and material assertions are unchanged; no new assertion or binary
sabotage is introduced. The current G7 alias controls remain. One setup
line is replaced by three; zero binary lines are changed or removed.
No machine, historical build, box, model turn, fixture matrix, flag, retry,
deadline or concurrency change is added. No elapsed savings are claimed.

Boxed `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, `cargo build --locked`
and diff checks passed before commit. No e2e, sabotage, host CI or runtime
setup ran in this box. This does not establish that the original 0/47 Mac
gate is resolved. Exact corrected source still requires full operator Linux
and Mac qualification; the separate historical startup cause remains unknown.
Prior archives and failures, Pinfold 0.2.6 pins, production code, construction
state, `.yard` and installed binaries are unchanged. No release readiness,
publication, installation or operational switchover is claimed.
