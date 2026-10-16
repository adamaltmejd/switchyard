# Y80 legacy project alias ownership repair

2026-10-16. Continues `54f7f9c673305f71b66d7f4e996261f9f5dac630`.
Review 2079 passed with P2 at daemon.rs:393. The operator classified it
as a migration and ownership integrity blocker. Supported direct POST
`/rpc` init previously accepted an absolute symlink spelling verbatim.
Registry startup retains that root and derives project/runtime ownership
from it. Canonicalizing only subsequent request paths left the loaded
legacy project inaccessible, allowed repeated init to use another identity,
and could report forget success while retaining the active legacy project.
This source defect is repaired; no new reproduction run is claimed here.

Resolution now compares the caller's absolute path with registered roots
and their current canonical paths. A unique match returns the recorded
root verbatim. Lookup, repeated init and forget therefore use the same
loaded project key, store root and runtime ownership as startup. Existing
roots are not rewritten, relabeled or migrated. With no match, new init
still uses a canonical root. Missing paths retain their exact absolute
spelling, preserving gone-root forgetting without guessing former alias
destinations. Canonicalization errors other than missing paths stop the
operation rather than ignoring an unresolvable registration.

Distinct registered roots that resolve to one directory are ambiguous,
even when the caller supplies one root exactly. They refuse requests with
an operator-resolution message. Startup validates the full registry for
these collisions before opening any project store or reconciling effects.
It does not choose one owner, open both and then guess, or rewrite history.
Repeated identical registry entries do not create distinct identities.
The absolute-path requirement remains at the daemon boundary, including
direct API calls; worker containment and effect recovery are unchanged.

The operator reports the existing G7 real-symlink positive passed on
`54f7f9c`; its deterministic literal-lookup sabotage failed sync with
`not_found`, and source was restored. This evidence remains bound to that
source. The existing representative is retained unchanged. No new machine,
legacy build, assertion, fixture matrix, box or model turn is added for
this correction. Boxed fmt, locked all-target clippy, debug build and diff
checks passed before commit. No host/e2e or sabotage run was performed here.

Prior source and failure archives remain unchanged. The original Mac
historical startup exit 1 is still under separate operator diagnosis.
Pinfold 0.2.6 and native Linux CI runtime pins remain. New source requires
fresh operator correctness review and Linux/Mac runtime qualification.
No full qualification or release readiness is claimed; no host runtime,
construction state, `.yard`, installation or operational switchover changes.
