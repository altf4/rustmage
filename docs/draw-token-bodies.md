# Complete draw-and-token bodies

Issue: https://github.com/altf4/rustmage/issues/160

Seven complete bodies bring executable coverage to **1,242/5,164**, with **3,922
unsupported**. Six regression identities, 56 token factories and one emblem
remain separate. Standard stays pinned to October 15, 2026.

## Complete bodies and provenance

| Card | Whole body |
| --- | --- |
| Biomechan Engineer | 2/2; entry creates a Lander; {8} draws two cards and creates a 2/2 colorless Robot artifact creature |
| Lunatic Pandora | Legendary artifact; {2}, tap surveils one; {6}, tap, sacrifice destroys target nonland permanent |
| Popular Egotist | 3/2; {1}{B} and sacrifice another creature or enchantment grants itself indestructible until end of turn, then taps it; sacrificing a permanent drains one from target opponent |
| Reckless Lackey | 1/2 with first strike and haste; {2}{R} and sacrifice itself draws one and creates a Treasure |
| Redrock Sentinel | Artifact 2/4 with defender; {2}, tap and sacrifice a land draws one and creates a Treasure |
| Seize the Spoils | Sorcery; discard a card as an additional cost; draw two and create a Treasure |
| The Sackville-Bagginses | Legendary 2/2; entry may sacrifice another creature or artifact to draw one and create a Treasure; sacrificing a token makes target opponent lose one life |

`draw-token-body-admissions.json` records all seven Oracle bodies and 58 SHA-256
entries across 34 unique Java sources at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Sources include all constructors,
entry and sacrifice triggers, discard/source/external costs, optional-cost
scoping, draw and token effects, damage-independent life changes, temporary
indestructible, tap, surveil and removal effects, targets, keywords, and complete
Lander/Robot/Treasure recipes. Each pinned Git blob was compared against the
read-only checkout. Existing token factories supply all recipes.

## Compiler changes and boundary

Permanent-program parsing now trims each line after reminder-text removal.
Removing a trailing reminder previously left a space before the newline, causing
otherwise exact multi-sentence activation bodies to fail compilation. This
normalization unlocks the complete Pandora and Egotist bodies. It preserves
unknown residual text and complete-body rejection.

An authenticated source name can use the printed plural `enter` spelling in an
entry trigger. This binds The Sackville-Bagginses to its exact source, with the
same incarnation rules as other entry abilities. An unrelated name does not
acquire that binding.

The exact draw-plus-token sentence composes existing instructions in printed
order: draw first, then create the fully parsed token. The conditional optional
sacrifice entry program skips both instructions when declined or unpayable.
Source-return aliases, pronouns, arbitrary conjunctions, unsupported tokens,
unknown trailing effects and alternate conditional scopes are not inferred.
Fanatical Offering's explore-capable Map, Dyadrine's mana-spent entry and
multi-object counter payment, delayed copies and other incomplete bodies remain
gated.

No production runtime instructions, mutable fields or continuation layouts were
changed. Snapshot schema 49, C ABI 1 and control protocol 2 remain unchanged;
initialized logical state stays 19,784 bytes. State-byte accounting excludes
capacity, allocators, COW sharing, immutable definitions, observations and
external snapshots.

## Rules and owned continuations

The existing engine enforces private exact discard versus public sacrifice
costs, source tap and summoning sickness, ownership/type/source exclusions,
optional payment, target legality, countering, indestructible expiry and legend
choices. A paid sacrifice/discard stays paid when its lower spell is countered.
Egotist's sacrifice queues its targeted life trigger above the pending protection
ability; neither protection nor tapping occurs during cost payment. Its old
ability cannot affect a later incarnation. Pandora's destroy ability sacrifices
its source before the target is destroyed. A legend-rule departure is not a
sacrifice.

Sackville's optional entry payment can use an eligible token, but cannot consume
its source. The resulting Treasure's later sacrifice is a distinct token event.
The token trigger looks back at frozen characteristics even if Sackville leaves
at the same time. Private draws, surveil and search retain owned choices and
observer filtering. A public bounced source remains known in hand; new draws do
not inherit that knowledge.

The [official September 25, 2026 rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt)
cover cost payment in CR601.2h/602.2, instruction order in CR608.2c, optional
resolution costs in CR118.12, independent abilities in CR113.7a, new objects in
CR400.7, sacrifice look-back in CR603.10a, legend cleanup in CR704.5j and temporary
effect expiry in CR514.2.

## Validation and Standard workload

Thirteen native tests cover both seats and all seven bodies, actual casts and
all abilities, complete token characteristics and Treasure/Lander abilities,
private draws/surveil/search, source/external/additional costs, summoning sickness,
wrong actor/type/source choices, optional payment and decline, targeted drain,
destroy versus indestructible, real cleanup, legend choices, exact incarnations,
and countered Seize with its paid discard preserved. Owned decision boundaries
are replayed through clone/restore with stale-action rejection. A token-generation
capacity failure restores the earlier draw and the entire pending resolution.

Three compiler tests protect exact draw/token composition and order, conditional
skip scope, reminder whitespace, authenticated names, full-body admission and
negative residuals. Existing historical fixtures now recognize the newly
implemented bodies and seventh mandatory spell additional-cost definition. One
fresh release Python test exercises eight scenarios, including both Pandora
abilities, through real native costs and cloned/restored spell/ability/entry
frames.

Complete validation: **985 Rust tests** (973 unit, 12 integration), **185
compiler/data tests**, **132 fresh release Python tests**, all-target Clippy with
warnings denied, formatting, catalogue consistency and diff checks.

Two Standard100 runs reproduced the same vectors in the table's order:

- 126,920 decisions; casts `[128,448,175,460,291,258,396]`.
- Body programs `[67,903,209,290,1275,258,396]`; finished programs
  `[67,903,204,290,1275,256,396]`. These count activated abilities for the first
  five bodies, Seize's spell and Sackville's entry trigger. Games can end with
  pending frames.
- Pandora's surveil/destroy activations `[632,271]`; Egotist/Sackville drain
  triggers `[592,626]`.
- 123,591 creature-history checks; 858 private choices, comprising 226 searches
  and 632 surveils; 46,685 clone/restore replay boundaries.
- 21,864 maximum initialized logical state bytes.

The separate 1,000-game native Freeform benchmark completed 571,049 decisions at
roughly 407,669 decisions/s and 714 games/s; clone averaged 109 ns and restore
98 ns. It includes policy and observation construction, excludes neural
inference/tensors/JSON, and is not an XMage parity comparison.
