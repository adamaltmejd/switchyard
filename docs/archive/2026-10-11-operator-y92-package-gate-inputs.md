# Y-92 host package gate inputs

The Y-92 package tests require public offline image-build inputs. The
operator reverified all115 locked npm SHA512 archives and both official
Node26.11.1 Linux archives. Only public data is stored under
/home/yard/validation/switchyard-durable-inputs, outside worker mounts.
No package, Node, model code or script was installed or executed on host.

The operator e2e gate supplies that path in its shell command. The gate
keeps its host placement, existing environment allowlist and30-minute bound.
The command invokes the whole real e2e suite; missing inputs do not skip
package tests. This configuration takes effect through supported sync.

Y-92 attempt82 admitted the old command before this package fixture
existed. Its frozen admission cannot inherit a later gate change, and the
operational daemon environment has no YARD_E2E_DURABLE_INPUTS. The daemon
will not be replaced or restarted. Its exact committed candidate
bebbbe4e6522a6289c424ddb24bde9db9a0a3d78 is preserved in a verified
public source bundle against870ba9d. The operator will park and abandon
that attempt through supported CLI commands, retain the full ticket scope,
and admit fresh work with the configured gate. The new worker imports the
public source in its own box and integrates actual Y-97 before a new
exact-head audit, host tests and review.

Y-97 actual landing1671 ended successfully with69/0 host tests and
cleanup1676 removed before this import. Sol6.1medium with fast tier,
the DeepSeek reviewer, manual approval, two lanes and all controls remain
unchanged. Package runtime acceptance is still unproven; this is input
configuration, not a passing package test or Y-92 completion.
