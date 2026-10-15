# E2E orphan helper audit

Read against validated baseline `d12e30c` after the test surface reduction.
The six removed helpers have no callers in the repository: distinct merge
dates, attention watch spawning, watch result parsing, attention extraction,
single-file Codex commits and oversized operator frames. The watch parser's
only caller is the removed attention extractor. Remove their unused `Read`
import too. The retained Codex file-list helper still serves its caller.

This removes 75 test harness lines and adds one import line: 74 net lines.
No scenario or exercised assertion changes. Every guarantee row and its
Shown by remains unchanged. No new coverage or replacement scenario is added.
The earlier risk-based scenario reduction remains a separate disposition.

Formatting, locked all-targets clippy and locked build pass. All-targets
compilation confirms retained scenarios need none of the removed helpers.
Native integration validation and final release qualification are pending;
this prepared commit is not evidence of a passing host suite.
