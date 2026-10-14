# Y150 first host failure: image adoption

The operator's first required host scenario failed at the image-adoption
count: `closed` ran seven times against the fixture's expected six. The run
took 59.01 seconds in the suite and 73.1641 seconds wall time. The reported
invocations were initial closed/other/unknown, bookkeeping unknown, proof
and tree closed/other/unknown each, command adoption closed, runtime
declaration closed, then two closed and other/unknown at the image boundary.
Earlier explicit-reference assertions reached this boundary. Later policy,
accepted-failure, skip and landing assertions remain unqualified.

The invocation log contains gate names without execution IDs. The failed
host store is unavailable in this worker. It cannot establish whether the
two closed commands had identical consumed inputs. Two reported scheduler
configuration-load messages have no demonstrated correlation with them.

Source inspection traces `sync` through canonical import and publication,
then a separate `yard_attempt_revise` adopts the image source. Gate launch
loads its execution's frozen revision. Verification records the candidate,
producer binding, frozen definition and observed image before running the
command. These paths identify the evidence needed; they do not prove an
intermediate invocation, changed producer/base, or repeated matching input
in this failed run. No product or source-adoption change is justified yet.

The existing G2 image count assertion now prints at most the latest 48
execution identities, only on failure. Its SQL projection includes safe
execution fields, candidate producer IDs, reuse references, tree identities,
the frozen operator-failure flag, image/context blobs and frozen source
commits. It omits commands, environment, credentials and full result records.
The projection includes implementation rows so producer IDs can be resolved
to their own candidate and revision. The count remains six.

Test-audit change gate: the retained block shows G2's image-adoption
invalidation clause. Its existing sabotage comment names omission of image
matching and rerunning unrelated consumers. The expected invocation counts
remain fixture-owned. No new assertion or scenario is added; the change
only exposes evidence when that existing assertion fails. Zero assertion
lines and zero binary lines are removed. No deadline, concurrency, routing,
unknown-input control or prior node-policy repair changes.

The operator retains this first failure and runs the existing independent
scenario after the diagnostic change. Its execution identities must decide
whether to repair the product or the controlled source-adoption fixture.
There is no blind retry or claim of feature completion. Host e2e is not run
inside this worker; boxed fmt, clippy and build are the worker checks.
