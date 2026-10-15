# switchyard

[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) is the spec, and it is binding.
`docs/archive/` is dated history, not a spec. This file holds the working
rules for changing the repository.

## Rules

- Implement the spec. A design change updates ARCHITECTURE.md in the same
  commit. Follow-up work is a ticket, not a TODO in code.
- Living docs (README.md, ARCHITECTURE.md, this file) state the current
  contract only, tersely. Evidence, measurements and rationale go to a new
  dated file, `docs/archive/YYYY-MM-DD-topic.md`. Archive files are never
  updated to track the present.
- Build the smallest thing that meets the spec. No speculative abstractions,
  compatibility paths, or speedups that haven't been measured to matter.
  Delete rather than keep.
- The controls in ARCHITECTURE.md (the threat model, the guarantees G1 to
  G15, the box table, the worker-tool grants) are exact. Weakening one is a
  spec change, never an implementation detail.
- Jobs, not states. Every long-running thing is an execution on the one
  state machine. No feature gets its own lifecycle, owner, lease, receipt or
  recovery path.
- One current truth. Readiness, the current check, approval, liveness and
  merge readiness each have one authoritative reading.
- The daemon is the only store writer. Operator MCP/API talks to the daemon; git and
  pinfold are child processes with explicit cwd, a scrubbed environment,
  bounded output and time.
- No secret value in argv, logs, the store, or any file Yard writes into a
  box.
- Dependencies are the ones ARCHITECTURE.md lists. A new one needs a reason
  in its commit message.
- Plain prose, short sentences, in docs and comments. Comment only
  non-obvious intent, footguns, issue links and revisit triggers.
- A ticket is admitted for a new guarantee, a bug reproduced through the
  supported operator MCP/API, or a consolidation of one module that changes no test assertion and
  names what it removes: a concept, a path, a special case, a duplicate.
  Fewer lines is the usual evidence, not the gate; a consolidation that adds
  an abstraction and removes nothing is rejected. A proposal born in an attempt
  is rejected unless it names the guarantee or bug it serves.
- Before each release, the `code-cleanup` skill with `test-audit`'s sweep
  (`.agents/skills/`): a whole-tree read for yagni, duplication,
  wrong-altitude fixes, unmeasured cost and assertion blocks under the bar,
  landed one commit per module; then the operator's spec pass over
  ARCHITECTURE.md and README.md. The pass reports the release's delete/add
  ratio, the largest source file and ARCHITECTURE.md's line count in a dated
  `docs/archive` file.

## Tests

- **End-to-end only.** No unit tests, mocks, runtime stubs, or test-only code
  paths or flags in the binary. A test builds the real binary, starts a real
  daemon on a real project with real pinfold boxes, and observes from
  outside: service exit codes, structured operator MCP/API output, the store file, canonical git state,
  host files, or acts from inside a box through `pinfold box exec`. The e2e
  crate never imports yard's internals; its only seams are a user's: service/MCP startup,
  supported operator MCP/API, `config.toml`, `operator.env`, the git repositories, and the
  tools a worker calls.
  Historical setup may use an explicitly selected real old executable's CLI
  when it lacks successor operations. No version-detecting compatibility
  dispatcher or successor task CLI is supported. Closed history fails event
  waits promptly; an independent held FIFO may outlive planned daemon death.
- **A small core.** Select tests by the cost of failure: unauthorized effects,
  stale identity, corruption and recovery, ownership, isolation, secrets, and
  representative end-to-end use. G1 to G15 are product requirements, not a
  demand for exhaustive automated proof. A unique case does not earn a test
  by itself. Secondary options, configuration combinations, lifecycle
  permutations and historical versions may remain untested.
- **Adapt before adding.** Change or replace an existing representative test
  by default. Secondary behavior may stay untested. New permanent fixtures are
  exceptional. Remove obsolete assertions and paths; judge the whole suite's
  machines, historical builds, boxes and model turns, including added phases.
- **Representative hard cases.** Prefer one difficult case over several easy
  variants of the same mechanism. Count actual fixture machines, builds, box
  launches and model turns; combining tests does not reduce that work.
- **External evidence.** Each retained assertion has an observer outside the
  binary and an expected value from the spec, Git, pinfold, fixture or host.
  Yard outputs may be compared when agreement is the promised behavior.
  Nothing tests argv shapes, file layout, help text or log prose; a reason
  is a spec-named token. Keep assertions focused on the selected failure.
- **Coverage is a choice.** The guarantees table's Shown by column describes
  the current representative suite. It is not an inventory of cases that
  must all be tested. Remove low-value coverage and update that description
  together. Record deliberately untested behavior tersely in a dated archive;
  do not create replacement tests or tickets merely to restore coverage.
  Product controls and promised behavior remain exact.
- **Tests must be able to fail:**
  1. Positive controls. Every "refused" test shows the allowed version
     succeeding in the same project.
  2. Assert the reason, not only the failure: a stale result plus the
     identity it names. A timeout is not a pass.
  3. Names its sabotage: the change to the binary that makes it fail, in
     the test's comment.
- **Deterministic.** No sleeps: consume bounded `events` pages with an ordered `since` cursor. No
  retries: a flaky test is a bug to fix or delete. No test waits out a
  timeout or a clock; timeouts are configuration, not guarantees. The
  workers are real Pi in real boxes against a scripted fake model on the
  host, reached through a pinfold route; the script decides what the "model" commits,
  proposes, publishes or refuses. No public endpoint is called.
- **Crashes are deterministic.** A crash test kills the daemon at a point
  that holds until it is killed: a row the store shows while a box the
  fixture is holding runs, or a git call the daemon is making. Git is found
  on the daemon's `PATH`, so a wrapper there can run the real command and
  then kill the daemon, or kill it first and hold the command until the
  test releases it. Seeing a row or a ref is not holding a point. Before
  restarting, the test proves from outside the state it meant to reach. Where
  no such point exists the scenario is not written; the binary grows no
  barrier for it.
- **Where it runs.** On a host with the container runtime, never inside a
  box. It is this repository's landing-stage host gate, on yard-sthlm and
  the operator's Mac, after the boxed candidate gates `cargo fmt`, `clippy`
  and `cargo build`. GitHub runs it on Linux x86_64 and arm64 for every push
  to `main` and every pull request. macOS runs on the operator's Mac before a
  release.
- **Budget:** about 5 minutes per landing on yard-sthlm with a warm image
  cache. Record measurements in a dated archive. Runtime and fixture cost
  inform coverage choices; there is no scenario-count target.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
```
