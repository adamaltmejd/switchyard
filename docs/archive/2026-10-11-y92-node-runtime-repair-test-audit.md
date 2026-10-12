# Y-92 Node runtime repair

The operator ran the focused g08_durable_package host gate at 1b2ec61.
Host execution 81197 ended with exit 101: zero passed, two failed, 14.84s.
Both isolated image builds verified the official Node 26.11.1 x64 archive
checksum, then the ELF loader refused Node because libatomic.so.1 was absent.
No outer Pi, controller or model request ran. This failure does not establish
the missing-assets test's refusal or its corrected-image control.
The operator's log is /tmp/switchyard-y92-1b2e-package-host.log on the host;
this worker did not read that host path.

The measured image failure establishes Node's missing library requirement.
Read-only inspection in this candidate found Debian 13 and dpkg metadata
identifying libatomic1:amd64 14.2.0-19 as the provider of
/usr/lib/x86_64-linux-gnu/libatomic.so.1. readelf reports its SONAME as
libatomic.so.1 and its NEEDED library as libc.so.6. This installed version is
an observation, not a new invented image pin. The candidate includes gcc
from the repository's existing apt-based recipe; its installed packages do
not prove that the default base already contains libatomic1.

Fetching the official v26.11.1 BUILDING.md from raw.githubusercontent.com
failed with curl exit 22, HTTP 403. No official-source verification or Node
execution is claimed from that operation. The repair uses the operator's
actual image failure and the candidate's package-provider evidence.

The Durable Dockerfile now installs libatomic1 with apt from the reviewed
Debian base's authenticated repositories during isolated image build, before
install-image.sh first invokes Node. apt selects the target architecture and
required OS dependency closure. The base continues to supply its C/C++
runtime. No manual library copy, host installation, controller-time package
install or runtime download is added. The README and binding specification
name this OS dependency and its reason. When appending the scaffold to an
offline image build, the operator supplies a reviewed base already containing
the closure and omits the OS installation RUN. No offline .deb input was
required or fabricated in this repair; authenticated OS repository access
during the operator image build remains to be exercised.

The official Node archive checksums, Node 26.11.1 pin, all scoped package
pins, exact npm lock and pre-model identity checks are unchanged. The lock
SHA256 remains
38d2d7a889d74d6e1863ea2a2be86b5ca8ed0ab9e1cef0b288195e6fe12ab33e.
Production bridge and controller source are unchanged. Repository .yard and
historical archives are unchanged. G8, G12, G13 and all Y-97 acceptance
clauses remain intact; daemon selection still awaits Y-93.

## test-audit disposition

Read the test-audit skill. This repair changes no test or assertion block.
Retain both g08_durable_package scenarios and all eight retained mechanism
groups recorded in the preceding integration audit: preflight, declared reads,
report byte bounds, trusted instructions/code exclusion, secret containment,
permissions, iteration bounds and explicit native publication. No observer,
positive control, deadline, harness command or expected value changed.
The operator's failing build is repaired at the image dependency boundary,
before the existing scenarios can exercise their mechanisms. No guarantee
is weakened and no new test seam is added.
Test lines removed: 0. Binary lines removed: 0.

cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings and
cargo build --locked passed. Shell syntax and git diff --check passed.
No e2e or image build ran inside this candidate. No host Node, library,
controller or package script ran; no host package was installed.
Fresh exact combined source audit, focused packaged-image host tests, the
full 71-scenario host suite, current review and actual landing gates remain
required. This scaffold repair is not runtime acceptance.
