# Independent review of PR #6

Two subagents reviewed the foundation PR on October 15, 2026. Reviews were
read-only and independent of the implementation work.

## Rules and rollback review

Scope: the documented 14-card slice, including priority, London mulligan
sequencing, mana payments, target validation and resolution, combat declarations
and division, cleanup, rejected actions, and complete snapshots.

Result: no actionable findings within that scope. The reviewer ran all 19 Rust
tests. This review does not certify omitted mechanics or full Magic conformance.

## Library and binding review

Scope: Rust API, C ABI ownership and safety, Python lifecycle/concurrency,
revisions, observation privacy, and protocol errors.

Finding: **P1, malformed configuration could abort the embedding process.** A
decoded NUL in an unknown card name reached a plain-text error message. Converting
that message to a CString panicked outside the FFI catch boundary.

Fix: escape embedded NULs before constructing every returned C string. Added
`test_decoded_nul_in_invalid_card_is_an_error_without_aborting_host`, running the
input in a subprocess so a future regression is detected without aborting the
test runner.

The reviewer rechecked the fix, reran the release build and all four Python
tests, and confirmed that the original reproduction now raises `EngineError`
instead of terminating the interpreter. No remaining confirmed findings.

## Validation

- `cargo fmt --all -- --check`
- `cargo clippy --offline --all-targets -- -D warnings`
- `cargo test --offline`: 17 engine tests and 2 protocol tests.
- `cargo build --release --offline`
- `PYTHONPATH=python python3 -m unittest discover -s python -p 'test_*.py' -v`:
  four tests against the native shared library.
- `cargo run --release --offline --example branch`: native branching example.

Benchmarks and limitations are recorded separately. Both reviewers explicitly
limited conclusions to the accepted catalogue and trusted in-process API.
