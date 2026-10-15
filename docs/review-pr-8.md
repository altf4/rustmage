# Independent review of PR #8

Two subagents reviewed the stacked catalogue/admission PR on October 15, 2026.

The rules reviewer inspected every newly admitted card's costs, characteristics
and complete rules text, the six keyword implementations, Standard copy limits,
rejection paths, and preservation of rollback. No actionable findings.

The library/data reviewer checked the full catalogue, Java reconciliation,
generator, importer, serde/C ABI/Python behavior and privacy. The reviewer
regenerated the inventory byte-for-byte and confirmed the recorded source gaps.

One **P2** finding: absent `oracle_text` defaulted to an empty string in the
creature-template compiler, so incomplete future data could silently turn a
triggered creature into vanilla. The current frozen snapshot had no incorrect
admissions. Fixed by requiring explicit string text and rejecting absent/null
text during normal-card/face import validation. Added template regressions,
import regressions and validation across the complete frozen corpus.

The reviewer verified the fix and all 11 script tests; executable Standard
coverage remained 26 cards. No remaining confirmed findings.

Final validation:

- 26 Rust tests: 20 engine, 4 catalogue, 2 protocol.
- 11 importer/compiler/inventory tests.
- 6 Python tests against the release shared library.
- Formatter, Clippy with warnings denied, release build, generated-file checks.
- A complete supported Standard self-play example terminated naturally.

Review conclusions apply to the implemented rules and catalogue boundaries.
All-card semantics and broader rules remain incomplete; knowing every identity
or passing corpus/admission checks is not proof of all-card rules correctness.
