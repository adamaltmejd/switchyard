# Y-57 review repair

Review blocked c4014a867ff7a3f10d2eb212b985bd949d55c65c with four source
findings. These findings are not new host runtime observations. This repair
changes queue and admission code and the binding contract. Earlier dated
audits remain historical and unchanged.

## Reserved landing publication

Capacity admission already waits when validated default input is absent.
The later queue policy admission still refused that absence. Target input
Git reads between reservation and publication locking left a window for a
concurrent default mutation to invalidate input. The reserved landing then
became an error despite a legitimate mutation progressing on another target.

Queue policy admission now waits on project events when publication is
absent. It registers notification before reading under the publication lock,
drops that lock before waiting, and rechecks its execution and live attempt
on each wake. Its existing execution.waiting event names the execution.
The existing reservation, target lock and inherited host lock stay owned
until the enclosing effect completes and proves release. No slot is guessed
released and no second reservation or lifecycle is created. A capacity
waiter before reservation still reserves nothing. Abandon wakes the queue
waiter through the store transaction; the existing row outcome remains
authoritative. Publication remains unavailable during uncertain ownership.
Default selection still serializes with the transaction that writes intent.

## Sync branch resolution

Unnamed sync formerly derived its target from a default read before waiting
for locks, then validated it against the default read after locking. A
concurrent selection could make those identities differ. Sync now reads
default under publication after acquiring its target lock. If an unnamed
selection differs, it drops both locks and repeats lock acquisition for the
current branch before any import or configuration validation. This is a
lock-selection loop, not a retry of a failed Git mutation.

Named sync still refuses its exact open intent before waiting for the target
lock. All syncs recheck intents and retained landing ownership under their
selected locks. An unnamed sync does not refuse an intent on a stale
pre-lock default. Ordinary imports still release publication before their
Git ref effect; default classification and invalidation remain under
publication. Completed false CAS and uncertain command errors retain their
different publication treatment. No blanket error refresh was added.

## Queue eligibility and attention identity

The queue marked a target busy before skipping a red-blocked approval.
That left later eligible approvals blocked even without a running landing
or intent. Busy targets now come from recorded running/reserved work and
open intents, plus the landings actually started in this queue pass.
Red-blocked, withdrawn and superseded approvals do not consume a target.
Approval order among eligible candidates on each target remains intact.

Attempt start formerly inspected only the first stopped/red attention item
before attempting default refresh. A named intent item later in that list
could not recover unavailable input. Start now selects the exact requested
item after the existing stale-identity check. Without a name, it prefers
an open-intent recovery item on the ticket. Recovery still uses the existing
target-bound ownership and ancestry proof, and resolves only that item.
Other start paths still require validated default publication.

## Validation and limits

No e2e source, assertion or scenario changed. All 52 host scenarios remain
required, including the existing one-slot continuation and the independent
target cap, intent, ordering and CAS observations. This repair adds no
claim that those scenarios passed, nor that the four source races were
reproduced by the worker. Host e2e and runtime sabotage were not run here,
as instructed. Fresh exact-head source review and the full unchanged host
suite remain gates before approval.

cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings and
cargo build -p yard --locked pass. The Git I/O heap pin, normal runtime
settings, persisted intent ref, shared default-configured cap and bounded
proof are retained. No dependency, test-only path, operational process,
configuration or .yard file changed. No development daemon was started.
