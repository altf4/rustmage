# Independent review of PR #228

Reviewed frozen head: `0e0f181f789a85b665af833c7c4087ec82ddd372`.
PR: https://github.com/altf4/rustmage/pull/228

Two independent nonauthors reviewed scalar X in separate worktrees and verified the exact GitHub head. Runtime suites used checkpoint `6247da8946e24cc796b8405a528e21c4986993fa`. The final delta contains compiler guards and regression tests, generated fingerprint metadata and documentation; generated gameplay code is byte-identical after fingerprint normalization. Changed compiler paths were independently rechecked. This record changes no runtime behavior.

## Rules/compiler — `/root/tuck_acceptance`

Approved with no remaining findings. Independent native validation passes 1,452 tests. The complete 281-test compiler suite passed before the scoped fixes; four scalar/boundary methods, four bargain methods, 150 independent adverse mutations and the catalogue check passed after them. The review found three compiler admission gaps: later bargain selectors revived rejected X costs, malformed fixed bargain costs became free, and overflowing scalar fixed bases became free. All were fixed and independently rechecked. The representable maximum-u32 fixed base remains supported; base-plus-chosen-X arithmetic remains widened at runtime.

Fresh-release Standard100 reproduced all vectors and 192,778 decisions/replays, 617 exact domains and 1,234 illegal rejections, with 19,976 initial and 23,536 maximum logical bytes. Independent public instrumentation covered 64 choice frames, 314 alternate concrete forks, 66 atomic out-of-domain rejections and four legal values absent indexed representatives. Both-viewer privacy and replay held.

All twelve complete constructors and 75 recursive source records across 50 pinned Java files were audited. Canonical comparison preserves 5,246 other definitions, all 74 token factories and their metadata, ordinary identities/colors and all 504 previous typed helper arms; thirteen helpers are added.

Actual evidence uses prefix `/tmp/issue224-rules-review-`: `native.log`, `compiler.log`, `compiler-fixed.log`, `bargain-fixed.log`, `compiler-probe.log`, `catalogue.log`, `release.log`, `standard100.log`, `public-probe.log`, `provenance.log`, `constructors.log` and `canonical.log`.

## Library/API — `/root/independent_library_189`

Approved with no actionable findings. Independently passed 1,452 Rust tests, 190 fresh-library Python tests (241.048 seconds), Standard100 with the exact 192,778-decision ledger, Clippy, formatting and catalogue checks. Final compiler source was separately loaded from the exact frozen Git commit: bounded-cost regression, reciprocal X/bargain gates and twelve positive controls pass. Generated runtime equality apart from fingerprint was independently proved.

Expanded public probes passed 64 scalar frames, 404 legal forks, 832 atomic wrong-actor/malformed-JSON/domain-hole/boundary rejections and 64 child actions after closing parent handles. Sixteen concrete values absent bounded representatives completed actual mana payment and Mind Spring draw effects. Exact X, complete sorted/disjoint domains, actor privacy and original-parent isolation held. The independent canonical/provenance audit agrees with the rules review.

Python used an immutable explicitly selected finished release library, SHA256 `d56b77b3ef15e26b4b1f62ff9d44f9540a1aa94a6dcb678aeb5de1d6f4cf4516`. One subprocess test resets PYTHONPATH and initially used the byte-identical freshly built default library; it also passed a separate rerun with explicit stable-library injection. No mapped shared library was overwritten.

Actual evidence uses prefix `/tmp/issue224-library-review-`: `rust.log`, `release.log`, `python.log`, `standard.log`, `probe.log`, `compiler-probe.log`, `final-compiler.log`, `semantic.log`, `java.log`, `child.log`, `clippy.log`, `format.log`, `catalogue.log` and `verdict.log`. Exact PR metadata is `pr.json`; probe and semantic scripts use the same prefix.
