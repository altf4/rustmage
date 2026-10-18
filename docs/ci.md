# Continuous integration

The **CI** workflow runs on pull requests using one `ubuntu-24.04` VM and Rust
1.98.0. Code changes run formatting, pinned-source provenance checks, strict
Clippy, every native test target (including Foundations full games), a release
build, the existing benchmark smoke workload, and seven Python smoke tests.
The Python checks cover schema agreement, handle lifecycle, card admission,
invalid input, hidden observations, stale actions, independent branches,
snapshot restore and a complete game. Existing tests are selected by name in
`scripts/python_tests.py`.

Changes containing only Markdown or LICENSE run a whitespace check and skip
compilation. Machine-read provenance JSON under docs/ still runs the full job. New
commits cancel superseded runs for the same PR. The normal job timeout is 20
minutes, with a 15-minute validation-step limit to leave room for diagnostics.
There is no duplicate push workflow or platform matrix.

All compilation and Python simulations use `scripts/resource_guard.py`. The
runner initializes a systemd user manager, then verifies the existing 6 GiB
parent/child cgroup limits, no swap, one Cargo job, one test thread, and serialized
builds. No allocation probes deliberately trigger the OOM killer. A runner that
cannot enforce those constraints fails before compilation. XMage is fetched at
the inventory's pinned revision for provenance tests; Java is not compiled.

To measure the full Python suite, open **Actions → CI → Run workflow** and select
`full` for `python_suite`. This manual job runs native validation plus full Python
discovery with a 50-minute validation-step limit and 60-minute job ceiling. Full Python tests never run automatically.
Each test writes a flushed start/finish timing record, so an interrupted run
still identifies the unfinished test. An always-run cleanup stops remaining
systemd build processes before collecting diagnostics. Until this suite is measured, its total
runtime and current passing status remain unverified.

CI retains diagnostic logs, resource reports, validation results and Python
timings as `ci-diagnostics-*` artifacts for seven days, including on failure.
It caches downloaded Cargo dependencies, not build outputs or shared libraries.
The validator cleans completed disposable outputs and preserves its exact
shared libraries for the lifetime of the VM. Benchmark timings are diagnostic,
not performance pass/fail thresholds on shared runners.

The same commands work locally:

```sh
python3 scripts/validate.py --release --python smoke
# Opt in to the potentially long full Python suite:
python3 scripts/validate.py --release --python full
```

The binding accepts `RUSTMAGE_LIBRARY` to select a built library. An explicit
`Game(..., library=...)` overrides it; without either, the normal
`target/release` location is used. The validator supplies its own library path,
including to tests that start child Python processes.
