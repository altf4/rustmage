# Observation schema version

`data/observation-schema.txt` is the single positive decimal observation-schema
version. Rust reads it at compile time; the Python binding exports the same value
as `rustmage.OBSERVATION_SCHEMA`. Current-contract tests and examples import that
constant. A schema change therefore edits one version file, alongside its actual
interface changes, instead of rewriting inherited policy assertions.

Historical delivery documents, immutable source/library evidence and preservation
proofs retain their original versions. The shared constant describes this checkout;
passing an older incompatible native library still fails the observation-version
assertion. It does not adapt a policy to missing fields or waive compatibility.

Validation consists of a native wire-version check, a Python FFI observation check
against an explicitly selected library, and a source check against repeated numeric
versions in current Python assertions. No card behavior or schema bump is included.
