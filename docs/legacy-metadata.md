# Supplied metadata and reserved fixture identities

Issue #276 closes the old name dispatch bypass. The fourteen reserved IDs remain stable for Rust constants and absent demonstration fixtures. When a record is supplied, its full rules body, types, stats and costs must pass the same generic compiler used for any other name. A reserved name does not grant implementation status or authorize ignored text. A missing rules field cannot select the absent-fixture fallback.

Basic-land status comes from supplied `Basic Land` type metadata. In particular, a creature named Forest is a creature with the normal deck limit. Genuine basic lands retain their authenticated intrinsic mana reminder and subtype matching.

The compact `TapMana` action for the original basics and Llanowar Elves is a representation choice after generic admission. The compiler compares the exact fully parsed activation cost and single fixed-mana instruction against the compact representation. A changed mana color, extra life cost, different stats or unsupported residual cannot restore the old behavior. Intrinsic keywords such as haste remain part of the compiled definition. Other abilities, static tables and costs retain their parsed semantics.

## Scope and evidence

This change adds six compiler test methods covering all fourteen reserved identities with changed complete creature, damage spell and land bodies. The controls check exact stats, keywords, type bits, mana cost, target/amount, basic status and deck limits. Negative controls cover bare or appended Discover, other unknown clauses, malformed types/costs/stats, unknown layouts and missing rules metadata. Mana controls exercise changed production, life payment, haste and complete-body rejection.

The composed candidate preserves all 5,303 generated definitions, colors, helper tables, token recipes and emblems from merged main `7027e1ef3972a5e2ceca1a47287c6b1c78f5dec2` byte-for-byte except the fingerprint. Eight supplied legacy records report their actual generic template provenance instead of `explicit`. Coverage remains 1,817 of 5,164 Standard identities, with 3,347 still gated; all thirteen Omen pairs remain admitted. No runtime state or observation fields are introduced by this change.

Frozen composed runtime/generator source: `45b01469809f2a08daa5441533db08f7545adf88`. Later merge `146a4c845d64de2d87da0a1ae72fa3960230c44f` synchronizes final main documentation without changing source or generated data. Fingerprint: `e9f2ea75b154fcfed1b1e69757352ca63009a32bbb698d6b9c28bd2e848dbfc7`; inherited schema 80. Its optimized immutable library is `legacy276-composed-45b0146-e9f2-8c1d3f1c1e61-schema80.so`, SHA-256 `8c1d3f1c1e61cbc65e5d5568c2ba94eae15b13460a0514bb2a8f8de649379128`.

Composed native validation passed 1,703 tests: 1,691 unit tests in 5.66 seconds plus 12 integration tests. All-target Clippy passed with warnings denied (14.07 seconds), as did formatting. The optimized library built in 31.32 seconds. Ten unique focused fresh public API methods passed in 64.640 seconds: six core binding/rollback/mana/Standard-combat checks and actual paid timing, selected-X, one-graveyard and split-search cases from the four composed Omen families. Every constructor, including the NUL subprocess, asserts the exact library fingerprint and schema. The complete composed compiler gate is still running with the explicit pinned XMage source configured; final approval awaits its terminal result.

The earlier standalone source `beb1c10904150275b0297426c3629390d0e2fdf5` also passed all 369 unique compiler methods (701.812 seconds), all 237 unique fresh public API methods (1110.506 seconds), 1,678 native tests and all-target Clippy. That evidence belongs to the earlier 5,300-definition/schema-79 catalogue, fingerprint `bed0fbb1e029d6acb7f9662eb523368f3ce3c07cec6555f5f630dd2c02fcd91d`; it is not presented as a full public-suite rerun of the composed catalogue. The emitted runtime programs remain canonically identical to the respective parents, so the parent Omen delivery's full runtime acceptance is retained at its own immutable fingerprint rather than rerunning unchanged hundred-game workloads solely for the fingerprint.

All completed native/release build targets have had disposable non-shared files removed immediately after their jobs ended. Every shared-library path and content is preserved and SHA-256 verified. Source, reports, logs and active compiler tests remain intact.

## Remaining reminder audit

This phase does not close #261. Inherited generic templates still erase arbitrary parenthetical text. Removing the legacy-name bypass makes supplied names obey those same existing templates; it does not prove those templates have closed reminder grammars. Unknown parentheticals and modified known reminders require the separate mechanic-by-mechanic audit tracked there. The strict timing, Adventure, Omen, Saga and attachment Ward routes retain their existing raw-body checks.
