# Y80 final qualification, 2026-10-16

Y80 is done. Approval224 landed candidate
`11a6cf689b566f96919737be5028478bd7cf0e46` as
`7c5817d98bb46290a080258da1d56a57787c6c2e` in execution3968.
`landing.recorded` is event13914. Supported sync consumed the landing.
The earlier user documentation commit `a67caa0` is included. Y81 stays parked.
This is release readiness for Yard0.0.5, not publication or operational switchover.

## Qualification

Every row passed all47 real native scenarios, with49 model fixtures and444
scripted requests. Tests use real daemons, Git and Pinfold boxes with the host
fake model. No public model endpoint, new timeout, retry, concurrency reduction
or weakened ownership/containment assertion was used.

| Run | Source | Suite seconds |
| --- | --- | --- |
| yard-sthlm qualification | 11a6cf6 | 537.19 |
| Linux x86_64 CI38951199748 job117142705087 | 11a6cf6 | 547.29 |
| Linux arm64 CI38951199748 job117142705267 | 11a6cf6 | 434.64 |
| Mac CI38951402019 job117143400142 | 11a6cf6 | 494.91 |
| Selected landing host gate3972 | prepared7c5817d | 591.46 |

Linux jobs explicitly checked out the reviewed commit. The first combined
run's Mac setup failed before tests because an operator script reused an input
destination that must be new. That failure remains; only the temporary workflow
was corrected and Mac rerun separately. The combined run is still failed.
Prepared7c5817d differs from11a6cf6 only in the committed user AGENTS guidance
and historical cleanup checkpoint. Landing fmt3969, clippy3970 and build3971
passed before e2e3972. Its full15453-byte log matches the store's terminal tail.
Landing native wall time was606.319s. Its raw SHA256 is
`cc727b7f0fde20d9b84bd759085acc464e6d1a2fb680cc975c79bf0979fb2338`.

The qualification host observed280 container creations and280 matching
removals, then an empty private inventory. Read-only retention observed52
project paths and714 database snapshots; aliases can name one physical database,
so these are not independent machine counts. Host raw log SHA256:
`5e33350e1b327813483b6df2dd808ed0cd2455836e5c7851673b0398d7db6a91`.
The host and landing runs exceed the indicative five-minute budget. No causal
speedup is claimed from cleanup; fixture and runtime cost remain visible.

Detailed source-bound results and raw paths:
`/home/yard/validation/coordinator-20261014/y80-11a6cf-final-platform-qualification-20261016.json`
and `y80-final-landing-verification-20261016.json` beside it.
Host evidence:
`/home/yard/validation/y80-11a6cf-final-full-026-20261016-evidence`.
Landing evidence:
`/home/yard/validation/y111-operator-evidence-20261013/y80-11a6cf-approved-landing-20261016`.

## Audit and recovery

The whole-tree code-cleanup and test-audit sweep, subsequent changed-source
refresh, and operator ARCHITECTURE/README pass are complete. Module commits
remove the unused session copy, duplicate execution JSON merge and repeated
input conversion. The stale-approval, registered-alias ownership and real
migration/restore corrections remain functional changes.

At qualified11a6cf6 versus Y80 base8cbcef: whole tree1777 additions/554 deletions
(delete/add0.31176); production94/73 (0.77660); e2e515/386 (0.74951).
Versus verified0.0.3 source81da8c6: whole release59945/4170 (0.06956),
production16732/1793 (0.10716), e2e11020/1900 (0.17241).
These endpoints exclude later operator documentation. Largest production source:
`crates/yard/src/jobs/admit.rs`,3305 lines. ARCHITECTURE was2632 lines at the
qualified source and is2634 after documenting the actual Mac prerequisite.
Full metrics: `y80-11a6cf-source-metrics-20261016.json` in the coordinator directory.

The representative suite decreased from51 scenarios/52 model fixtures to47/49.
Coverage choices are in the candidate cleanup and observer archives. The latest
G5 correction waits for the actual execution's checked release and interrupted
end, then retains all original store, box, tree, continuation and provenance
assertions. Only its exact intermediate ownership attention is allowed. No
fixture, wrapper, FIFO, helper, box or model turn was added. A temporary actual
binary sabotage that recorded `unchanged` instead of `interrupted` failed
directly on the ended-event outcome. It was reverted before the full pass.
The genuine0.0.3 upgrade, separate unupgraded restore, exact historical decisions,
held effects and no-duplicate reopen cases passed on every required platform.

## Apple native builder fix

Stock Apple container1.5.0/builder-shim0.13.1 corrupts concurrent equal-hash
contexts by sharing writable receiver staging. The retained failed archive has
duplicated4MiB chunks. Removing duplicates recovers a valid archive and all92
expected file hashes. The operator independently rechecked this evidence.

The fix creates transfer-private staging and removes it after joined senders
finish. BuildKit layer caching remains unchanged; cross-transfer unpacked
receiver reuse is removed. No dependency was added. Local upstream fix commit:
`ee6142f7066aad53b7878bf4bab4aade45ee6214`, based on
`e18d2182fd060dbf1c68113a74e7564d563dde27`, at
`/home/yard/validation/apple-builder-fix-20261016`.
Supplied full patch SHA256:
`77f48c21db678529c676726d4fdad46686b370e72196255525ae2b054fae75e1`.
The changed method in `pkg/fssync/diffcopy.go` is:

```go
func (f *FSSyncProxy) DiffCopy(ss filesync.FileSync_DiffCopyServer) error {
	ctx := ss.Context()
	if err := os.MkdirAll(f.basePath, 0o755); err != nil {
		return err
	}
	// Equal context hashes must not share writable transfer files.
	transferPath, err := os.MkdirTemp(f.basePath, "transfer-")
	if err != nil {
		return err
	}
	// run waits for the walk and every file sender before returning.
	defer os.RemoveAll(transferPath)

	fs := NewFS(ctx, f, f.contextDir, transferPath)
	s := &sender{
		conn:         &syncStream{Stream: ss},
		fs:           fs,
		files:        make(map[uint32]string),
		sendpipeline: make(chan *sendHandle, 128),
	}
	return s.run(ctx)
}
```

Build identity: Go1.25.12, Linux arm64, CGO0, vendored dependencies,
`-trimpath -buildvcs=false`, tags `osusergo netgo static_build seccomp`,
ldflags `-w -s -extldflags '-static' -X main.VERSION=y80-transfer-private-e18d218`.
The running shim SHA256 is
`74dd3465d99db67db2837d37a2b86f5fd5fd68b34d7757819703223b4f6a53a2`.
It is installed in runner-local image
`localhost/yard-y80-builder:transfer-private-74dd3465d99d`, descriptor
`sha256:9d5dc387a98905d239f81f7f5e4569690b9b48fee698d67f83b1f427180b244a`.
Its immutable image base is the stock builder descriptor
`sha256:b5b3f7fa81e662db6929f1ad66d835d151a1b03f682cfe5f9fcb17fa46d6bcc9`.
Only `build.image` in the runner's user/effective Apple configuration changed.
The builder was replaced with user authorization; BuildKit's executable hash
remained unchanged. Both full Mac runs verified the actual running shim hash
before and after qualification. Yard itself was not installed or replaced.

Native controls retained the original2 large failures and6 small successes.
The patched service passed8 simultaneous fresh contexts, including equal
payload metadata with differing Dockerfiles, under default cache/TMPDIR and
original deadlines. All190 copied hashes and per-image Dockerfile results
matched. An intentional failed build followed by a fresh unchanged-payload
transfer verified all92 recovery hashes. No transfer-owned staging remained.
The first short-lived image reader returned1 after printing correct bytes;
that failure remains. Held-owner inspections verified all8 actual images.
Mid-transfer cancellation and isolated extraction overhead were not tested.
Full Yard runtime timings above measure the qualified workload, not a causal
microbenchmark. This qualifies the patched shim, not unmodified stock0.13.1.

Investigation, source snapshots, patch and corruption proof:
`/home/yard/validation/coordinator-20261014/mac-native-build-context-investigation-20261016/report.md`.
Installed native fix evidence: `y80-apple-patched-native-validated-evidence-20261016/report.json`
in that coordinator directory. No upstream issue/comment/release was published.

## Pins, prepared assets and handoff

Pinfold0.2.6 source is `3d49304b9c55a7a8605c870db02f178d7c35c985`.
Verified Linux x64 and Mac arm64 release executables were used. Linux uses
Podman6.1.4 source `464c15fec06f0074c916f9eff87c4806869c2833`.
Hosted CI uses native overlay; yard-sthlm keeps its separate FUSE validation
store. FUSE is not a new Yard requirement. Rust1.99.0, Node26.11.1 and
Durable SDK1.2.0 retain their independent verified pins.

Three version0.0.5 assets are ready at
`/home/yard/validation/coordinator-20261014/y80-34f315-0.0.5-prepared-dist-20261016`.
Their recorded build source is `da68880f43de8c104f359d5370a325fc432b8e4f`.
Production code, Cargo manifests/lock, toolchain, embedded skills and Durable
runtime inputs are identical to11a6cf6; differences are CI, three test files
and documentation. Every binary hash was reverified. Build provenance is not
relabeled. Proof: `y80-11a6cf-prepared-artifact-input-equivalence-20261016.json`
in the coordinator directory. The final operator commit changes only docs.

Before a future operational Yard switchover, stop and reap the sole writer,
prove effects quiescent or externally held, and take a consistent copy of the
store/WAL, registry, canonical and required live artifacts. Use the migration
switchover archive with its recovery correction. Never open upgraded state with
an old binary or force-release unknown ownership. Old software need not be
retained under the user's authorization; irreplaceable project data must be.
No Yard release, main/tag push or operational switchover occurred here.

Earlier source/runtime/observer failures remain. The old x64 startup message
collapses all flock errors; its original errno/holder was not captured. Current
passes do not identify that holder or resolve Y147/Y148, Pinfold89 or Mac100.
Y81 remains deferred. These limits are not erased by passing qualification.
