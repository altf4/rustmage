# Controlled-land untapped entry — PR #343 / issue #342

Complete **Spelunking** and **The Wandering Minstrel** compose actual main2812c749.
Coverage is **1,979/5,164 Standard identities**, with3,185 gated,97 token recipes,
one emblem and schema105. Full Standard remains incomplete.

Entry transactions now retain the original event separately from effects replacing
entry with tapped or untapped entry. The incoming controller chooses the order of
competing effects; each exact source applies once. Shockland life payment remains
a real choice within its selected replacement. A plain instruction putting a land
onto the battlefield tapped is changed by an applicable untapped replacement, while
an intrinsic tapped-entry replacement may be applied afterward. Sources entering
in the same batch do not affect other entrants. Temporary owned entry state supports
clone, parent drop, restore and atomic failure; no permanent gains another field.
These distinctions follow the [official LCI release notes](https://magic.wizards.com/en/news/feature/the-lost-caverns-of-ixalan-release-notes)
and CR614.12/616 in the [pinned rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).

Spelunking draws, offers the existing private optional hand-land choice, and gains
life only for the selected Cave's actual successful battlefield entry. Its callback
retains the resolving controller through source departure and suspended entry
choices. Minstrel includes its five-Town intervening combat trigger, complete
all-color2/2 Elemental, and fully paid eight-mana Town-count boost to other creatures.
The boost uses the existing resolution-time quantity and cleanup machinery.

The [source inventory](untapped-entry-admissions.json) pins88 records/52 full Java
files across four direct users and relevant imports. Gond Gate and Horizon Explorer
are absent from this Standard corpus and are not counted as admissions. Strict
actual-parent preservation retains5,331 other definitions, all928 old helpers and
all96 old token recipes. Only two definitions, their untapped helper arms and one
all-color Elemental factory/alias are added. The existing compatibility grammar is
retained for this completed batch; subsequent migration uses the direct Rust
card-authoring path requested by the user.

## Validation

Production7872b98 passes **1,922 unit+12 integration tests (1,934 native)** and
all-target Clippy. Nine paid native groups cover both seats, both replacement
orders, shockland payments, source/control changes, multiple sources, simultaneous
entry, full normal bodies, real Korvold foreign casting, successful-entry callbacks
and owned capacity-failure rollback. Independent reviewer probes additionally
exercise joint Worldsoul shockland batches and a paid Disenchant response to a
foreign Spelunking trigger.

Review found an unsupported draw-land tail could raise a Python exception. Corrected
bc89f78 rejects unknown subtypes and oversized life amounts; all24 affected compiler
methods pass in5.423s. Every valid generated program, helper, color, alias and recipe
is identical to7872b98, with only the compiler fingerprint changed. No shared Rust
engine behavior changed for that correction. Earlier factory-count, cleanup-step
and exile-cast test-fixture failures remain failed diagnostics; the corrected
1934-native run is the passing claim.

The corrected immutable schema105 library builds successfully in30.730s. All18
existing naturally drawn, paid Standard cases pass in18.005s (18.251s driver),
covering both complete bodies, Cave/non-Cave/decline, both tapped-entry orders and
shockland payment branches in both seats. All932 execution inputs and the library
hash stay fixed. Formatting and generator checks pass. The final remaining-family
metadata regression also passes. This delivery does not claim a new full legacy API
suite or100-game bot workload.

All jobs are closed; terminal build outputs were cleaned with every shared-library
path/content preserved. Two independent nonauthor exact-final-head reviews gate
merging. Three complete hand-land consumers remain tracked in issue #330.
