# Y-92 package input blocker

This candidate implements no controller and adds no runtime guarantee.
Y-92 remains incomplete. ARCHITECTURE.md still says Durable is not runnable.

## Evidence

The candidate starts at `870ba9d1876fccde2a79783633114823169555e5`.
Y-91 supplies the validated report payload and execution-scoped MCP session
contract in ARCHITECTURE.md and `g08_audit_reports.rs`. These are sufficient
publication contracts, but do not define the Durable library API.

No supplied package artifacts or reference source were found in the
checkout, `/yard/input`, `/yard/proof`, or the searched installed files.
The earlier Y-64 candidate `23faf5d23cc14861aa6927ff74b8804e270e6988`
contains only its scope assessment. It explicitly reports the missing
prototype and does not verify package APIs.

Requests to `https://registry.npmjs.org/pi-durable/1.0.4`,
`https://registry.npmjs.org/pi-ai/1.0.4`, and
`https://registry.npmjs.org/chord/1.0.4` each failed with HTTP 403.
Separate HEAD probes to `https://registry.npmjs.org` and
`https://github.com` failed at the proxy CONNECT with HTTP 403.
These refusals establish unavailable access, not absent packages. The
unscoped probe names do not establish the actual package coordinates.

An exact lockfile needs verified package coordinates, registry metadata,
integrity digests and transitive dependency resolution. Reviewed source
needs the actual exported API and checkpoint behavior. Inventing those
would not meet Y-92's exact runtime/bundle identity requirement.

## Proposed prerequisite

Preserve the full ticket and supply the reviewed package coordinates,
Node patch identity and artifacts through trusted build-time access or
operator-provided input. Include the three 1.0.4 package archives, their
complete transitive closure and registry integrity metadata, or a verified
lockfile with its artifacts. Supply the referenced prototype if its API
contract is needed. The worker can then review the actual libraries and
produce source, lockfile, scaffold and boxed acceptance scenarios.

This prerequisite serves Y-64's explicitly requested G8/G12/G13 guarantee;
it is not a request for host installation or runtime network discovery.
Installed `.yard` image and configuration changes remain operator-owned
through commit and sync. Daemon selection remains Y-93, and automatic
checkpoint resume remains Y-65.

## Verification boundary

The test-audit skill was read. No assertions, guarantee rows or test
helpers changed, so there are no new assertion blocks to gate or delete.
The existing publication tests do not prove packaged controller behavior.
No host e2e suite ran, as instructed. No model request, package installation,
development daemon, image change or `.yard` write was performed.
`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` both passed.
