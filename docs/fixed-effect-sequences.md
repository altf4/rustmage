# Fixed damage subjects and effect sequences

Issue: https://github.com/altf4/rustmage/issues/180

Twelve complete bodies bring executable coverage to **1,341/5,164** pinned
Standard cards, with **3,823 unsupported**. Six non-Standard regression cards,
57 token factories and one emblem remain separate. The full Standard migration
remains in progress, pinned to October 15, 2026.

| Card | Complete body |
| --- | --- |
| Callous Inspector | 1/1 menace; death deals one damage to its controller and creates a Clue with its complete two-mana/sacrifice/draw activation |
| Cosmium Blast | Four damage to a targeted attacking or blocking creature |
| Deadly Riposte | Three damage to a targeted tapped creature, then gain two life |
| Defibrillating Current | Four damage to a targeted creature or planeswalker, then gain two life; all eight exact two-brid payments retain printed mana value six |
| Fear of Lost Teeth | 1/1 enchantment creature; death deals one damage to any target, then gain one life |
| Joust Through | Three damage to a targeted attacking or blocking creature, then gain one life |
| Keep Out | Choose four damage to a targeted tapped creature, or destroy a targeted enchantment |
| Lightning Helix | Three damage to any target, then gain three life |
| Ravenous Giant | 5/5; its controller's upkeep deals one damage to that controller |
| Risky Shortcut | Draw two, then each player loses two life |
| Romantic Rendezvous | Discard one, then draw two; discard is an effect rather than an additional cost |
| Viashino Pyromancer | 2/1; entry deals two damage to a targeted player or planeswalker |

`fixed-effect-admissions.json` records all twelve complete Oracle bodies and
**65 SHA-256 entries across 41 unique Java files**, pinned to
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Every constructor was read in full.
The manifest includes damage/life/draw/discard effects, event classes, targets,
the combat and planeswalker filters, intrinsic menace and the existing complete
Clue token/activation. Recorded blobs match the pinned revision and checkout.

## Closed subjects and ordered programs

The fixed-damage parser uses a closed subject vocabulary. Existing subjects
retain their exact target rules. Tapped creatures and attacking/blocking
creatures reuse existing live target predicates. Player-or-planeswalker uses
one existing union target rule, with one target occurrence; it excludes
creatures. Creature-or-planeswalker continues to exclude players.

Optional controller life gain is emitted after damage. Existing multi-sentence
tails retain their original complete parsing, including damage followed by
draw, removal or life gain. Unknown targets, effects or residual sentences gate
the whole card. Damage and life amounts retain the existing bounds of 32,767
and 65,535; negative or untracked quantities are not approximated.

Every target is checked at declaration and resolution. Required-target
unavailability prevents casting, and all-illegal targets stop the entire spell
or ability, including its life gain. A new target incarnation cannot substitute
for the declared object. Ward counters the whole selected spell when declined.
Keep Out commits a legal mode before its own targets and cannot switch modes
when a target later becomes illegal.

Source damage retains last-known source facts, including lifelink after Fear
of Lost Teeth dies. Controller damage is distinct from life loss. Risky
Shortcut draws before both life losses; state-based losses occur after the
program. Romantic Rendezvous discards before drawing and exposes that choice
only to its controller. With no card available to discard, it still draws two.
Private frames and exact references survive clones and restores.

All behavior uses existing immutable instructions, target predicates/unions,
announcements and resolution frames. Production Rust changes are limited to
generated definitions. No mutable state field/layout, snapshot cache,
observation schema (**51**), C ABI (**1**) or control protocol (**2**) changes.
All previously executable definitions remain unchanged.

## Validation

Fifteen native tests cover actual full casts from both seats, all eight
two-brid options and printed mana value, both Keep Out modes, required target
filters, committed attack/block targets including an untapped blocker, exact
target revalidation/fizzle, player versus planeswalker/creature targets,
planeswalker loyalty damage, Ward, private discard ordering and source
independence, empty-hand draw, simultaneous life losses, last-known lifelink,
Clue's complete activation, owned-upkeep conditions and atomic overflow rollback
of damage, histories, source spell and revision. Semantic actions compare
entire cloned states and restored replay, including stale revisions.

Three compiler tests cover all fixed subjects and bounds, effect ordering,
unknown clauses, all twelve complete admissions and the existing damage bodies
whose multi-sentence tails must remain supported. One fresh release Python
method runs thirteen scenarios: every complete body, both Keep Out modes,
Clue activation and private discard replay, with independent public effects.

Two deterministic Standard100 runs use three legal 60-card/four-copy deck
families and reproduce:

- **105,597 decisions**, casts
  `[253,210,220,244,301,229,288,218,322,172,177,200]` in table order.
- Death/entry/step triggers
  `[220,0,0,0,284,0,0,0,504,0,0,200]`.
- Independently verified full effects
  `[217,210,220,244,283,229,288,217,498,164,177,195]` and **216 Clue activations**.
- Keep Out modes `[203,85]`, **1,260 attack declarations**, **439 fundable
  combat windows**, **108,226 public stat/ability pairs**, **163 private
  choices**, **29,501 replay boundaries** and **21,992 maximum initialized
  logical state bytes**.

The witness manually models damage and target validity, life gain/loss, draw,
discard, destruction and tokens using public before/after facts. It checks
every battlefield creature's public stats/abilities for both viewers without
reading private instruction definitions. Terminal-producing effects are
excluded from resolved-program counts. The smoke policy stops tapping mana
once a payment is available, retains unused lands across steps and holds
combat removal for opposing attackers/blockers. This makes those spells an
explicit workload acceptance requirement.

Complete checks: **1,124 Rust tests** (1,112 unit and 12 integration), **215
compiler/data tests**, **145 fresh release Python tests**, all-target Clippy
with warnings denied, formatting, catalogue consistency and diff checks.

The separate 1,000-game benchmark completed 571,049 decisions at about 411,602
decisions/s and 721 games/s, clone 110 ns and restore 101 ns. Initial logical
state remains **19,784 bytes**. Timing depends on the host, includes policy and
observations, excludes inference/tensors/JSON, and is not an XMage parity
comparison. Logical accounting excludes capacity/allocator overhead, COW
sharing, immutable definitions, observations and external snapshots.
