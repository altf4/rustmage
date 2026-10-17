# PR 284: ownership and foreign-library play

This is the first ownership/controller stage of issue #265. It completes Korvold and the Noble Thief and the currently admitted controller-sensitive paths. Broader changing-control bodies, their durations and layers remain gated; issue #265 and the all-Standard goal remain open.

## Frozen source and library

- Runtime source: `71b46f8c7242a14ca2143d4cfb0a2be3cfa8a449`, composing actual merged main `0977111f57734f6e4e22bf6c3c68a88358d29da9`.
- Explicit optimized library: `/home/altf4/.cache/rustmage-immutable/ownership284-71b46f8-4cf0-schema82-346fbe1f7bc0.so`.
- SHA256: `346fbe1f7bc073d77ebc93572e0198dbe0adcc583e8e1103b8887777803e9b23`.
- Catalogue: `4cf0215661aa81cc431c52f3a9039a9ea891d870acbc40b4387d714b9b254ebb`; observation schema 82.
- Coverage: 1,826/5,164 Standard identities, 3,338 gated; 5,311 generated definitions, 87 token recipes, one emblem, all 27 Adventure and 13 Omen helpers.

Canonical comparison with main preserves 5,310 definition rows exactly. Only ordinary Korvold changes from unsupported to its whole Saga body. Remaining generated changes are the provenance/version fingerprint and enabling that ordinary card in `Card::ALL`; no other helper, token recipe or identity color changes.

## Validation status

The frozen source passes all 1,724 native tests: 1,712 unit and twelve integration, including twelve controller groups and an independent 768-case physical-state funding/evidence oracle. All-target Clippy passes with warnings denied; formatting and catalogue generation checks pass. The optimized build completed in 42.88 seconds. All terminal build targets had non-library outputs removed immediately, with every shared-library path and SHA256 retained.

The full compiler/provenance and public API checkpoint gates started from `71b46f8`. Compiler collection has 403 unique methods with the pinned Java checkout enabled. Public collection has 249 unique methods; every constructor and NUL subprocess is forced to the exact optimized library and checks schema/catalogue. Those collected inputs precede the subsequent policy-only change described below; their result must retain that checkpoint scope. Compiler inputs and production Rust remain unchanged. Final results and two independent nonauthor exact-head approvals remain required.

The initial Standard100 attempt finished 74 games, then correctly failed its combat-witness assertion: seed2877 P0 cast foreign Beorn, but the established army ended the game before Beorn could attack. The revised policy withholds other attacks until that explicit witness occurs; it changes no engine semantics. Fresh two-method public acceptance and the complete 100-game workload must use an isolated export of the revised policy with a file-hash manifest and the exact 346f library. The earlier attempt is not counted as a passing Standard100 gate.

The earlier C793 immutable library belongs to the `f826f789b41607e8ae3692517cf5772eaea2b41e` checkpoint and does not contain the final funding-key ownership distinction. Its inherited 247-method API job and two new paid public methods are historical evidence only. The latter pass in 18.301 seconds; Omen witness seeds are P0 2843 and P1 2841. P0 seeds 2840, 2841 and 2842 did not expose Bloomvine through the Saga in their natural games and were recorded as fixture failures, not engine failures or passing acceptance.

## Review requirements

Review independent ownership/control routing, casting actor context, last-known controller binding after source departure, generic and Adventure incarnation-specific permissions, physical-owner Omen/graveyard routing, land allowances and cleanup, funding ownership distinctions, observation privacy, and owned rollback. No author approval or partial acceptance substitutes for two independent nonauthor reviews of the final published head.
