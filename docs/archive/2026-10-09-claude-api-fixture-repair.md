# Y-54 host-gate fixture repair

The landing gate for candidate `7f8fdbd` ran 37 real host scenarios.
36 passed. The new direct Claude API scenario failed before any review:
execution 1 raised `stopped:unchanged`, with no new worker commit. The
existing Claude login containment scenario passed.

The new implementer script used `act`, whose opening predicate requires
the request's last message to have role `user` and not contain only tool
results. That predicate can choose the terminal reply before sending a
commit. The landing report does not contain the model request transcript,
so the exact request shape is not established by that report.

The repair uses the same completed-tool-result discriminator as the
passing Claude login fixture: send the commit when no tool result exists,
then finish after the tool answers. The scripted real harness still makes
the commit. No production branch, retry, wait or test barrier was added.

## Test-audit gate

Keep all G6/G8/G12 assertion blocks and their expected values from the
spec, fixture and external pre-call snapshots. The refusal control remains
successful API and login work in the same project. The hardest review case
remains an actual publication without stdout registration proof, followed
by a fresh proven seat. The existing Pi stderr-proof and Claude login
scenarios still do not cover this direct API route with stdout proof.

The fixture repair changes no assertion. The previously named sabotages
still apply: accepting simultaneous provider/login, placing the literal
key in the guest, omitting host injection, or accepting the unproven check.
The implementation observers remain CLI history, host model requests, the
store, canonical Git and the real guest credential probe. No clause was
weakened. The operator's list is empty. Four test lines were removed and
zero binary lines were removed.

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. No e2e or
sabotage run was made in the worker, as instructed. A successful host rerun
is still required; the repair does not claim runtime validation.
The adapter contract and binding architecture are unchanged. `.yard/`
was not edited.
