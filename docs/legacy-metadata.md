# Supplied metadata and reserved fixture identities

Issue #276 closes the old name dispatch bypass. The fourteen reserved IDs remain stable for Rust constants and absent demonstration fixtures. When a record is supplied, its full rules body, types, stats and costs must pass the same generic compiler used for any other name. A reserved name does not grant implementation status or authorize ignored text. A missing rules field cannot select the absent-fixture fallback.

Basic-land status comes from supplied `Basic Land` type metadata. In particular, a creature named Forest is a creature with the normal deck limit. Genuine basic lands retain their authenticated intrinsic mana reminder and subtype matching.

The compact `TapMana` action for the original basics and Llanowar Elves is a representation choice after generic admission. The compiler compares the exact fully parsed activation cost and single fixed-mana instruction against the compact representation. A changed mana color, extra life cost, different stats or unsupported residual cannot restore the old behavior. Intrinsic keywords such as haste remain part of the compiled definition. Other abilities, static tables and costs retain their parsed semantics.

## Scope and evidence

This change adds six compiler test methods covering all fourteen reserved identities with changed complete creature, damage spell and land bodies. The controls check exact stats, keywords, type bits, mana cost, target/amount, basic status and deck limits. Negative controls cover bare or appended Discover, other unknown clauses, malformed types/costs/stats, unknown layouts and missing rules metadata. Mana controls exercise changed production, life payment, haste and complete-body rejection.

On the pinned main catalogue, all 5,300 generated definitions, colors, helper tables, token recipes and emblem definitions are byte-identical to the parent; only the fingerprint changes. Nine supplied legacy records report their actual generic template provenance instead of `explicit`. Coverage remains 1,811 of 5,164 Standard identities, with 3,353 still gated. No new runtime state or observation schema is introduced.

Full compiler, native, lint and public API validation are in progress; this draft is not a merge approval.

## Remaining reminder audit

This phase does not close #261. Inherited generic templates still erase arbitrary parenthetical text. Removing the legacy-name bypass makes supplied names obey those same existing templates; it does not prove those templates have closed reminder grammars. Unknown parentheticals and modified known reminders require the separate mechanic-by-mechanic audit tracked there. The strict timing, Adventure, Omen, Saga and attachment Ward routes retain their existing raw-body checks.
