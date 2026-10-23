# Continuous integration

The **CI** workflow runs card and interaction assurance on pull requests with one
Ubuntu 24.04 runner and Rust 1.98.0. Markdown-only and LICENSE changes run a
whitespace check without compilation. New commits cancel superseded runs for the
same PR. There is one native library test executable and one small Python API
suite; no full-game discovery suite or build matrix runs on every change.

Validation checks generated catalogue linkage, pinned Oracle metadata and unique
XMage source hashes before compilation. XMage is fetched at the inventory's
pinned revision, but its working tree is not checked out and Java is not built.
One `git cat-file` process verifies the distinct source blobs referenced by card
provenance and shared dependencies, rather than repeatedly hashing the same
sources through separate development-era test modules.

The compiled phases are formatting, strict Clippy on all targets, the optimized
library test build, production release library/benchmark build, and the Python
API checks against that exact release library. The test profile uses optimization
level 1 without LTO. `cargo test --lib --no-run` builds the native suite once;
the runner then invokes exact scenarios from that binary, records each duration,
and fails on the first unsuccessful scenario. Production release optimization
and thin LTO remain unchanged.

Runtime limits are explicit:

| Work | Wall-clock limit |
| --- | ---: |
| One native scenario | 10 seconds |
| All native scenario execution | 30 seconds |
| Python API assurance | 30 seconds |
| One test or release compilation phase | 6 minutes |
| Validator, including all builds | 9 minutes in CI |
| Validation step | 10 minutes |
| Entire job, including setup and diagnostics | 12 minutes |

The local validator defaults to a 10-minute overall budget. Build time and test
execution time are reported separately. These ceilings are failure limits, not
promised runtimes. Use the measured phase durations in the retained report to
identify regressions; shared GitHub runners are unsuitable for tight
millisecond-level timing assertions. The release benchmark runs ten seeded
games by default and records simulation decisions/second and games/second. It
provides a small diagnostic sample rather than a statistical performance claim.

All compilation and simulations enter `scripts/resource_guard.py`. Its shared
build lock, one Cargo job/test thread, 6 GiB parent/child cgroup and no-swap policy
remain enforced. A runner without those limits fails closed. Routine checks do
not allocate memory merely to trigger the OOM killer. Timeout handling stops
and reaps the command tree before disposable outputs are cleaned; an always-run
CI step also stops the systemd build slice on cancellation.

Logs, flushed per-scenario JSONL events, JSON phase reports and resource reports
remain under `~/.cache`. CI uploads them as `ci-diagnostics-*` artifacts for seven
days, including failures. Only Cargo downloads are cached. Completed disposable
build products are removed while every shared library is retained at its exact
path and verified SHA-256 for the lifetime of the runner.

Reproduce the CI workload locally:

```sh
python3 scripts/link_catalogue.py
python3 scripts/validate.py --release --python smoke --timeout 540
```

For native changes without a binding or release-build check:

```sh
python3 scripts/validate.py
```

`RUSTMAGE_XMAGE_SOURCE` selects a relocated pinned-source repository; otherwise
the sibling `mage` repository is discovered. `RUSTMAGE_LIBRARY` selects a built
binding library, and explicit `Game(..., library=...)` takes precedence. The
validator supplies the exact library from its own run to Python and its children.
Do not edit input sources while a consuming validation job is still running.
