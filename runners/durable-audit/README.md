# Trusted Durable audit image

This package supplies the explicitly selected audit and cleanup controller for
a Yard-owned box. Yard freezes selected inputs and owns supervision and
checked teardown on its existing execution lifecycle. No process here creates
a box or reads the host.

The operator builds the target image and commits and syncs installed
`.yard` image/config changes. Workers change none of the installed `.yard`.
Use Node **26.11.1**, and the unchanged package.json and package-lock.json.
The lock pins the complete closure, including the three direct
`@earendil-works` packages and overridden `pi-telemetry` at **1.2.0**.
Durable provides the conversation and checkpoints, Pi AI the routed model
client and schemas, and Chord the context passed to Durable. Telemetry is
pinned because Pi AI depends on it; no telemetry exporter is configured.

Build on an isolated image builder. Place the official Node Linux glibc
archive for the target architecture and all locked npm archives under
`build-inputs/` and `build-inputs/packages/` respectively. The Node filenames
are `node-v26.11.1-linux-x64.tar.gz` and
`node-v26.11.1-linux-arm64.tar.gz`. Verify against the supplied official
node-SHASUMS256.txt. Supply npm archives whose SHA512 matches the lock.
`install-image.sh` runs only in the build container: it populates an offline
npm cache, runs `npm ci --offline --ignore-scripts`, checks JavaScript syntax
and seals every installed regular file. No host Node/npm is required.

Use the Dockerfile as a scaffold, with the repository as build context,
or append its COPY and RUN instructions to the reviewed target Dockerfile.
The scaffold uses a reviewed Debian glibc base with its C/C++ runtime and
installs `libatomic1` from that base's authenticated package repositories
during the isolated image build, before invoking Node. This supplies
`libatomic.so.1`, which the official Node archive requires. Preserve this
step when appending the scaffold. OS packages are image-build dependencies;
none is installed on the host or at controller runtime. For an offline build,
prepare a reviewed base containing this runtime closure and omit the OS
installation RUN when appending the scaffold.
Set `TARGETARCH` to `amd64` or `arm64`. Pin the reviewed base by digest when
preparing the installed image. The controller lives at
`/opt/yard/durable-audit`, outside the candidate workspace. Keep that path
image-owned and unavailable for candidate writes or overlay mounts. Its
launcher checks the sealed installed closure and Node binary before running
any package code. Missing or changed assets produce `runner_unavailable`
with the failed asset, before any model request. Runtime uses no npm,
package discovery, download, candidate entrypoint or candidate extension.

Real Pi registers the execution's native MCP session. Its native input hook
deterministically invokes
`/opt/yard/durable-audit/launch /yard/input/audit.json` inside that same box,
before outer model dispatch. There is no outer Pi model request. The child
uses anonymous file descriptors 3 and 4 for one report request and
reply. Pi forwards it through its existing native audit publication grant.
Neither the MCP bearer nor the session id enters the child. Direct shell
invocation supplies no publication bridge and is not the supported launch.
The box must carry Yard's existing injecting model route; only the guest
placeholder enters the model client. No upstream key is an input.

Commit and sync operator-reviewed Markdown and its declared regular-file
references. Select a packaged runner through a profile, preset or structured
operator MCP plan. A profile for an existing Pi agent `worker` is:

```toml
[profiles.durable]
access = "read-only"
outcomes = ["complete"]
[profiles.durable.outputs]
result = "report"
[profiles.durable.runner]
kind = "packaged"
package = "durable-audit"
agent = "worker"
instructions = "Audit the declared source against the selected skill."
skill = "skills/test-audit.md"
references = ["skills/references/policy.md"]
files = [{root = "source", path = "crates/yard/src/pi.rs"}]
```

Use `yard_attempt_start` with `workflow` naming an existing read-only workflow
whose implementer is `worker`, and
`overrides: {"implementer": {"profile": "durable"}}`. Only one read-only
`implementer` node with report output and `complete` is supported there.
The existing exclusive `implementer -> cleanup -> repair` graph also accepts
a packaged cleanup profile with `access = "write-allowed"`, report output and
`complete`. Keep the implementer and repair profiles ordinary. Cleanup shares
their source clone; repair alone produces the final candidate. Arbitrary
graphs and delegation remain refused. A read-only workflow with no seats may
also set `profile = "durable"`; `yard_ticket_new` with `workflow: "NAME"` then admits
that explicit selection. The profile is not inherited from the default.
Supported providers are `openrouter` and `opencode-go`. Declare regular files
under source `/workspace`, proof `/yard/proof` or trusted `/yard/input/trusted`.
Selected Markdown must fit 32768 bytes. Reads return at most 32768 bytes with
a truncation flag. Admission, identity, checkpoint and execution bounds are
specified in [Executions](../../docs/ARCHITECTURE.md#executions).

The model receives `read_declared`, `write_declared`,
`commit_source` and `yard_publish_report`. Writes and commits refuse with
`source_write_refused` unless the daemon's frozen cleanup launch explicitly
sets `source_write = true` under `package_launch = "source-write-v1"`.
Audits retain read-only source mounts and no source-write authority.
`write_declared` replaces one existing declared source file with at most
32768 UTF-8 bytes. It refuses links, multiply linked files, traversal, `.git`
and `.yard` paths. `commit_source` takes a nonempty message of at most 1024
bytes and commits only declared source paths with boxed Git. Each Git call
has an explicit cwd, scrubbed environment, disabled hooks, filters, fsmonitor and
signing, 32768 bytes per output stream and a 120-second bound. Cleanup must
leave a clean committed tree before publishing its report. Both tools are
unsafe to replay; a deliberate continuation after checked predecessor release
receives interruption instead of automatically rerunning a pending tool.
Source access grants no candidate publication or operator landing tool.
No shell or delegation tool exists.
Report fields and byte bounds match Yard's validated publication contract;
the daemon remains authoritative. Successful publication asks Durable to
terminate. A bare answer raises `missing_report`. Logs contain only named
completion/refusal and numeric usage frames, never SDK exceptions or model text.

Host acceptance is `cargo test -p e2e --locked`. Set
`YARD_E2E_DURABLE_INPUTS` to a public input directory with the official
architecture-specific Node tar.gz and `packages/` containing all locked npm
tarballs. The fixture copies them into its isolated build context, then
builds the packaged image. Real Pi runs its configured command, registers
MCP and invokes the explicitly selected controller through its native input
hook. The fixture requires every model request to belong to the controller.
Missing assets refuse before any model request; a corrected image controls that in
the same project. No fixture writes daemon-private launch inputs.
CI provisions this directory on both Linux architectures with
`sh runners/durable-audit/fetch-build-inputs.sh "$YARD_E2E_DURABLE_INPUTS"`
before running the suite. The path must not already exist. The script uses
host curl, jq, OpenSSL and sha256sum to fetch public archives as data and
verify the committed Node checksums and each locked npm SHA512. It runs no
host Node/npm, package installation or archive code. The image build still
verifies and installs its offline inputs inside the isolated builder.
Run it on the
container-runtime host after approval; source and boxed Rust checks alone
do not establish runtime acceptance. Pi's real registration, completion,
native report and checked teardown remain necessary for result eligibility.
