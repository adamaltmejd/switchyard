# Y-92 CI runner context scope repair, 2026-10-11

Review 937 P2 on exact 678ab9a1a4d2f768e0a105acda30ccf1b3ee85f3
identified an invalid job-level runner.temp expression in ci.yml. The
operator read the workflow and verified the primary GitHub context
availability table:
https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#context-availability

That table excludes runner from jobs.<job_id>.env and includes runner in
jobs.<job_id>.steps.env. The invalid job environment prevented dispatch of
the required real host e2e matrix. This is workflow gate integrity, not a
failure of the package controller or an authorization to weaken host gates.

Move only YARD_E2E_DURABLE_INPUTS to the fetch and full-suite step env maps.
Both use exactly ${{ runner.temp }}/yard-durable-inputs: the same supported
runner-local absolute directory within each matrix job. The fetch step
passes that environment value to fetch-build-inputs.sh; the e2e process
reads that same variable for offline image consumption. No GITHUB_ENV
mutation, new input transport or host package execution is introduced.

The primary-document context availability above is operator-verified
supplied evidence. Independent retrieval in this candidate of the official
page and official github/docs raw Markdown source both returned HTTP 403.
No independent successful document fetch is claimed. The repair applies
the exact supplied primary table distinction and preserves runner.temp's
existing directory expression. Local source wiring checks cannot prove
GitHub dispatch or actual arm64 execution.

The two matrix runners remain ubuntu-26.04 and ubuntu-26.04-arm. Verified
public archive fetch, exact official Node 26.11.1 checksum, all locked npm
SHA512 checks and package pins, offline isolated image installation,
full cargo test -p e2e --locked and existing timeout are unchanged. No
assertion, test scenario, production ownership guard or binding spec changes.
All revision 13 and earlier package/recovery/integration controls remain.
No dependency is added; existing dependency reasons remain binding.
All historical archives and repository .yard remain untouched.

Operator full72 session 61657/test 403110 on 678a was reported live when
this correction was requested. It is not observed, interrupted, changed or
restarted here. Its eventual result is evidence only for its exact head.
No result or passing history is invented. Prior missing-path P3 remains
uncorrected. No runtime prune/kill/restart, deadline/thread-cap adjustment,
reviewer/model/configuration change, deployment, push or publication occurs.

Validation passed: source checks found exactly two identical Durable input env
values, both nested under steps; fetch still receives the quoted variable,
e2e still reads it, and full-suite command and architecture matrix remain.
Cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings,
cargo build --locked, fetch-script shell syntax and git diff --check passed.
No e2e, network provisioning, image build or GitHub dispatch runs here.
Fresh changed exact-head source audit/checks/review/full host and successful
actual landing remain required. No previous host green transfers to this
changed workflow; local checks are not GitHub or arm64 acceptance evidence.
