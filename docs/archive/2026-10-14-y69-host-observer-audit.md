# Y69 host observer correction, 2026-10-14

The landing gate at 0338ba7 reported seven boxes where the five-profile
scenario expected five. Its preceding store observation found exactly five
writer executions. The box query used the presence-only dev.yard.project
label. The e2e harness shares the host pinfold runtime, so that query also
counts boxes from unrelated projects.

The G9/G13 five-profile assertion now selects the exact project label that
pinfold reports on a held writer's box. It still requires exactly five real
boxes while all five host-model requests are held. Each listed box is also
matched to its writer's recorded handle. The following per-writer checks
still require independent clones, indexes, sessions, proof and producer
settings. No capacity, deadline, isolation or publication
assertion is relaxed. Other box queries in this scenario match exact
execution handles and do not count unrelated boxes.

The test-audit gate disposition is to correct the observer's project scope.
The guarantee and its existing Shown by clause are unchanged. Sabotage that
shares a clone/index/state/proof path or drops a declared writer remains
named by the scenario and fails its retained observations. Five is supplied
by the fixture graph and its five held requests; project membership comes
from pinfold, not a copy of Yard's project-key algorithm. This is the same
hardest concurrent scenario, with no added twin. The existing two-writer
crash hold and external proof are unchanged.

One test line is replaced; no binary source is removed or changed. The
contract does not change, so ARCHITECTURE.md needs no behavior amendment.
The required boxed checks are cargo fmt --check, cargo clippy --all-targets
--locked -- -D warnings and cargo build --locked. Host e2e is deferred to
the operator's landing gate as instructed; this correction claims no new
passing host evidence.
