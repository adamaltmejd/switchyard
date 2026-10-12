# Y-99 operator validation and remaining work

Candidate a5899026429b601e4e73723c1a38fbdb0222af13 implements audit
filesystem isolation. The operator read the source, architecture, original
test audit and review check968. The ineffective host-path checkpoint probes
were rejected. The corrected candidate deletes those probes and retains the
actual mounted-state, checkpoint, proof, resume and cleanup observations.
Its fresh review1842/check976 passed without findings.

Exact candidate focused host validation passed the packaged isolation
scenario: one passed, zero failed, 71 filtered, in 125.06 seconds. Its log is
`/tmp/switchyard-y99-a589-isolation-focus-host.log`. The operator evidence
manifest records its digest. No full-suite pass is inferred from this case.

Approval103 admitted actual landing1853. Merged input
7a2bbfe1f7b011fca85481f0cbdb83997834b5b0 passed fmt1854, clippy1855,
build1856 and real host e2e1857. The complete suite reported 72 passed,
zero failed, in 692.62 seconds. The retained terminal detail is
`/tmp/switchyard-y99-1857-terminal.log`; it is a bounded output tail.
The legacy-planner and extended packaged-isolation cases both passed.

Supported history recorded landing6434, ticket.done6435, attempt.ended6436
and cleanup1864 removed6438. Supported sync consumed the exact merged head.
This is actual Y-99 completion. Y-100 can now implement explicit packaged
runner selection and supervision; Y-93 remains the full integration parent.

Host validation used the isolated Podman/FUSE profile and public Durable
inputs under the CT2015 backup hold. The 692.62-second result exceeds the
historical five-minute estimate. It does not establish a cause or authorize
timeout changes, concurrency limits or assertion removal. The release's
required final measurement and audit remain outstanding.

Y-95 is not done. Landing1833/host1838 failed71/1 in496.97 seconds at a G6
whole-row observation before checked settlement. Its source-only repair
retains all assertions and passed exact focused validation in10.65 seconds.
Landing1848/host1852 then failed71/1 in499.01 seconds: the old legacy planner
was refused with image-missing before reaching its first model hold. Its
reference disappearance remains unexplained. A diagnostic candidate preserves
the assertions and captures bounded failure evidence; no runtime fix is
claimed. Y-99's later pass does not explain or erase either failed Y-95 run.

The user authorized publication of the Pinfold runtime investigation. Issue89
was posted and its body verified: https://github.com/adamaltmejd/pinfold/issues/89.
It asks for native-overlay diagnosis without making FUSE mandatory. No other
publication, push, installation or operational daemon replacement occurred.
The daemon retains binary digest
05ae25832964703c81525e7c5f5be7a8148954517cfddf69ef9846953bf1ebb2.
Models and reviewer remain unchanged. The complete redesign through Y-80
remains required, including Y-82/Y-83; Y-81 stays deferred.
