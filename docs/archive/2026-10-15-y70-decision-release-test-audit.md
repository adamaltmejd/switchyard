# Y-70 current landing decision release

The operator's preserved native 5250431 run failed at final omission
acceptance. Landing 36 had released preparation at audit 205, reserved
again at 206, raised omission attention at 207, ended at 208 and released
at 209. Watch.find returned cached 205. The answer therefore reached the
existing ownership guard while the parent was still reserved. The retained
source, binary, log, terminal and read-only store snapshot remain operator
evidence; this candidate does not claim a successful native qualification.

Disposition: retain every assertion in both selected-candidate scenarios.
The five-profile scenario now requires the exact omission parent's release
sequence to exceed the current attention sequence. The selected failure
policy scenario applies the same bound wait before both landing repair and
landing acceptance. Watch.find can consume retained or future events, but
cannot mistake an earlier preparation release for the current release.
No sleep, retry, binary barrier, ownership change or new helper is added.

Read together: Watch.find/until; the existing sequence-bound G2 omission
controls; queue preparation, omitted-gate attention, child gate admission,
failed-gate attention and capacity effects; candidate-answer identity and
prove_predecessors; the selected failure-policy decisions and final
five-producer omission answer. The prior committed superseded return fence
and valid fresh-conflict control remain unchanged in this candidate.

G2 exact decision evidence and G5 checked ownership release are the
contract; G8's existing selected failure/omission clauses retain their
same-project successful repair/acceptance controls. ARCHITECTURE.md makes
explicit that a prior preparation release cannot satisfy a later decision.
The observer is the real CLI history stream. Sequence ordering and exact
parent identity compare two outputs, as permitted for an agreement
contract; fixture Git and producer scripts still supply candidate content
and expected outcomes. The waits introduce no new expected verdict.

The existing comments name product sabotage: omit selected choice from
landing attention, omit the landing failed outcome, or treat repair as
acceptance. Those successful decisions still fail under those changes.
The synchronization correction prevents an unrelated ownership refusal
from masking that evidence. Existing ordinary G2 sequence-bound waits
exercise different graphs; they do not synchronize these selected tests.
The repeated reservation of the same landing execution is the hard case.
No assertion, refusal control, limit, guarantee or history check is removed.
Test lines removed: 1 unbounded release predicate, replaced by the exact
parent and sequence predicate. Binary lines removed: 0.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings, and cargo build --locked passed. Native/e2e was not run in
this box, as instructed. Host qualification remains operator-owned after
review. No .yard project configuration changed.
