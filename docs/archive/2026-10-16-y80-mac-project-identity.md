# Y80 Mac project path identity repair

2026-10-16. Operator evidence: Mac native CI `38924896289` on exact
`b670ca0840c1891819696d21eb3712a1ffd8cd90` failed 0/47 in 42.51 seconds.
Fmt, clippy, build, profile and public inputs passed first. The runtime was
verified Pinfold 0.2.6, macOS 27.0.1 and stable Apple container 1.5.0.
The operator retains the raw evidence; this is a supplied observation,
not a run performed in this box.

Most 45 fixtures failed through supported operator MCP sync. The forwarder
canonicalized init's `/tmp/ye-91939-.../p` to `/private/tmp/...`, but later
calls supplied the original `/tmp/...` as `project`. The daemon looked up
that spelling literally and returned `not_found`. This was inconsistent
product identity, not a permissible Mac skip.

Path resolution now belongs to the daemon for init, project lookup and
forget. All require explicit absolute paths. Existing paths resolve to the
canonical filesystem path before the exact registry lookup or registration;
the MCP forwarder no longer selectively rewrites init and forget. Alias
resolution does not register another owner or bypass registered ownership.
A missing path retains its supplied absolute spelling so an operator can
forget a gone project by its recorded path. An unresolved alias does not
guess a former destination. Other canonicalization errors are invalid.
Init still requires a Git checkout and refuses linked worktrees. No registry
rewrite, historical identity migration, containment or lifecycle change.

Test-audit disposition, G6: adapt the existing G7 cancellation/grants
machine to use a real symlink alias after its ordinary project setup.
Repeat init through the alias and compare its project identity to host
`std::fs::canonicalize`; existing sync, ticket, refusal, cancellation,
approval and Git landing assertions then run through that alias. The
material failure is loss of supported operator access to the registered
project. Sabotage is the old asymmetric init canonicalization plus literal
daemon lookup, named in the test comment. The external expected path comes
from the host filesystem; existing Git/store oracles remain unchanged.
There is no new machine, historical build, box, model request, deadline or
concurrency change. Forget shares the reviewed resolver; no extra forget
fixture or coverage matrix is added. One test line is replaced with ten;
the product diff removes 21 lines and adds 23. These are source counts,
not elapsed savings. No sabotage or e2e run is claimed here.

The two real 0.0.3 migration fixtures separately reported box startup exit 1
before ready in this failed gate. The operator is investigating that runtime
startup. Old historical source and those fixture controls are unchanged;
this repair does not claim to resolve their failure.

Boxed fmt, locked all-target clippy and debug build, plus diff checks,
passed before commit. Full exact-source Linux and Mac qualification remains
required; prior passes qualify only their original sources and runtimes.
No e2e, host CI, runtime setup or Mac dispatch ran here. Pinfold 0.2.6 and
the native Linux CI runtime pins remain. Construction state, `.yard`,
installed runtimes and old evidence are untouched. Release readiness,
publication, installation and operational switchover are not claimed.
