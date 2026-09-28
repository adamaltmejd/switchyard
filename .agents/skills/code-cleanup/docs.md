Edit nothing. Run `cargo build --locked`, then read README.md (if it
exists), `target/debug/yard --help`, and `--help` for every verb and
subverb. Read them as a user who wants to use the tool: every sentence
should say how to use it, as briefly as possible. Compare them with
`## Daemon and CLI` in docs/ARCHITECTURE.md.

One line per finding:

`<file or command>: <tag> <what>. [-<N>]`

- `cut:` history, motivation, rationale, a reference to a ticket or
  release, or a restatement of the spec where a pointer to it would do.
- `shrink:` wording longer than its meaning. Quote the new text.
- `drift:` help or README disagrees with the spec or with the binary.

Error and log messages are out of scope. Help text lives in
`crates/yard/src/cli.rs`. End with `net: -<N> lines.` or, when nothing is
found, `Lean already.`
