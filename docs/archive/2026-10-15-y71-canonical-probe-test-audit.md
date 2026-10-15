# Y-71 held integration canonical probe

Integrity/test-Gate review1803 found that an unchanged canonical HEAD cannot
expose an extra integration-only canonical mount. Retain the existing G2/G9
integration scenarios and strengthen the held real integration-box observation.
No production code changes.

The changed block serves G9's integration containment clause in Shown by and
G1's canonical isolation contract. The host writes known fixture bytes to a
sentinel in the disposable native project's canonical directory. Real pinfold
exec in the held integration box checks that canonical's actual host path is
absent or inaccessible, attempts to read the sentinel, and attempts to overwrite
it. Each unexpected access fails the block. The same command must then write
the expected partial bytes to its own /workspace/partial.txt successfully.
The host verifies the sentinel bytes and the existing partial-file assertion
still verifies that allowed write before restart. Only the disposable fixture
canonical is probed; construction canonical is never opened.

Precise sabotage: add a read-write mount of that fixture canonical directory,
or its .yard/local parent, at its actual host path during integration startup.
The absence check fails even without a ref update. The read check also exposes
read access; the overwrite attempt exercises write denial. The expected host
path comes from Project::canonical, the sentinel and partial bytes from the
fixture, and observations from real box exec and host filesystem reads.
No mount arguments, mount specifications or broad filesystem scan are asserted.

G1's ordinary worker probe does not exercise integration startup's extra input
mounts. The previous integration canonical HEAD assertion observes only ref
mutation; source read-only checks observe different mounts. This held real
integration box is the existing hardest case: it has imported A and written
partial work while the host model holds its next reply. Its execution row and
host partial file still establish the crash point before restart. All source
read-only, held restart, partial-work reuse, C-only approval/landing, conflict
repair and final event controls remain. No scenario or lifecycle is added.

Disposition: strengthen one existing block; retain all other blocks. No
guarantee or assertion is removed or weakened. Test lines removed: two sabotage
comment lines, replaced with the precise mount sabotage. Binary lines removed:
zero. The binding G9 Shown by clause now names the actual-path probe.

Worker validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings, and cargo build --locked. Native e2e and sabotage execution remain
operator-run. No host run is claimed here.
