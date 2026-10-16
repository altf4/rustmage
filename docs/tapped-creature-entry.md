# Tapped creature entry replacements

Issue: https://github.com/altf4/rustmage/issues/142
Pinned Standard: October 15, 2026. Five complete bodies bring executable coverage
to **1,162/5,164**; **4,002 remain unsupported**. Six regression identities,
51 token factories and one emblem are separate.

## Complete bodies and provenance

`tapped-entry-admissions.json` records exact pinned Oracle text and eleven SHA-256
entries across nine unique Java files at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All five card classes and the referenced
shared tapped-entry, opposing-turn, untap and Ward classes were read.

| Card | Whole body |
| --- | --- |
| Diregraf Ghoul | 2/2 Zombie, enters tapped |
| Piranha Fly | 2/1 Fish Insect, flying, enters tapped |
| Daring Thunder-Thief | 4/4 Turtle Rogue, flash, enters tapped |
| Sleep-Cursed Faerie | 3/3 Faerie Wizard, flying, Ward {2}, enters tapped with three stun counters, {1}{U} untap ability |
| Eddymurk Crab | 5/5 Elemental Crab, flash, instant/sorcery-graveyard generic reduction, opposing-turn tapped entry, optional zero-to-two targeted creature tap ETB |

Whole-body gates remain authoritative. Disguise, speed, graveyard activation/
cast-event returns, improvise, wider shuffle/copy/control effects and Tonberry's
own-turn keyword condition remain unsupported. The new extractor accepts one exact
creature self-entry clause, retains all other text and rejects multiple clauses
pending explicit composition support. Unknown/conditional/wider counter text stays
in the residual body and therefore fails full compilation. No incidental bodies
were admitted.

## Rules and implementation

Entry replacement metadata now supports a permanent entering tapped only when
its controller is not the active player. The two-player engine currently keeps
owner and controller identical; control-changing effects remain gated.
Unconditional tapped entry reuses the existing `Always` metadata. Combined tapped/
stun text retains the exact initial-counter clause for the existing permanent
compiler. Only complete supported bodies emit this entry metadata.

The existing entry transaction evaluates tapped state before movement and commits
it together with initial counters. Casting alone does not apply entry state:
spells remain counterless and untapped on the stack, including staged payment.
A countered spell never receives its entry state. Casting, Zombify, raw batch
entry and other existing zone-entry paths use the same replacement machinery.
There is no intermediate untapped/counterless battlefield state.

The existing stun replacement removes one stun counter per attempted untap while
leaving the permanent tapped. Three removals precede the fourth successful untap
for a fresh Sleep-Cursed Faerie. Both natural untap steps and the paid untap
ability follow this rule. Leaving the battlefield clears counters; actual
Unsummon and recasting produce a new incarnation with all three initial counters.

Crab uses the existing cost-reduction count over the controller's instant/sorcery
cards in their graveyard. Creature cards and opposing graveyards do not count;
only generic cost reduces, with costs locked at announcement. Its ETB chooses
zero, one or two distinct creature targets, independently of whether it entered
tapped. Ordinary sorcery timing and existing flash permission are unchanged.

No mutable state or observation fields were added. Schema 49, C ABI 1 and control
protocol 2 stay unchanged, as does **19,784 bytes** of initial logical state.
Snapshots include entry transactions, targets, counters, payments and exact object
identity. Restoration invalidates stale indexed actions. Catalogue fingerprint:
`90a3af94b4edfde4746afe251ec1fe8aaff5c09d00009fa4ee3cf343e1744ccc`.

Rules references: CR 122.1d (stun replacement), 400.7 (new object), 601 (casting),
603 (ETB triggers), 608 (resolution), 614.1/614.12 (entry replacements), and 702.8
(flash), from the September 25, 2026 text linked by the
[official rules page](https://magic.wizards.com/en/rules).

## Verification and performance

- **883 Rust tests**: 871 unit and twelve integration. Six new methods cover both
  owner/active-player relations, stack versus entry timing, flash legality,
  initial tapped/counter state, natural/paid stun replacement, actual bounce/recast,
  Zombify and simultaneous mixed entries, exact locked graveyard reduction,
  optional target cardinalities, illegal targets and countered spells.
- **168 compiler/data tests** include exact extraction/type/layout/multiple-clause
  gates, whole residual bodies and all five pinned admissions. Historical Crab
  negative admission expectations were updated for its now complete implementation.
- **121 fresh release Python tests**, including six staged-payment/entry scenarios
  across the five bodies and Crab's two turn relations, clone/restore, owned ETB
  targeting and stale-revision rejection.
- Format, all-target Clippy with warnings denied, catalogue consistency and diff
  checks pass.

`cargo run --release --example standard_tapped_entry -- 100` plays two legal
60-card Standard decks. It requires every new body, both Crab entry conditions,
stun removals, completed untaps, paid untap activations, noncasting entries and
clone/restore replay. Alternating policy seeds hold Crab for own/opposing turns;
odd seeds preserve untapped mana sources for flash. The workload also prefers
paid Faerie untaps. These preferences do not restrict the engine's action space.

Two runs reproduced **45,952 decisions, 105,818 body/entry checks, 411 stun
removals, 249 completed Faerie untaps, 112 paid untap activations, 15 noncasting
entries, 12,777 clone/restore replay boundaries and 21,420 maximum initialized
bytes**. Entry counts in table order were 137/90/186/185/143; Crab entered 69 times
on its controller's turn and 74 times on the opposing turn. The checker requires
exact tapped/initial-counter state on every new incarnation and exact per-attempt
stun removal on existing incarnations. Native rules scenarios cover the complete
individual spell/activation/ETB effects in addition to the workload.

An unpinned 1,000-game initial Freeform smoke benchmark recorded 571,049 decisions,
404,649 decisions/s, 708.61 games/s, 139.32 ns clone and 107.61 ns restore. It
includes native decisions/observations and excludes inference, tensors, JSON and
Python. This is a bounded smoke measurement, not Java parity or a full Standard
throughput claim. Logical byte counts exclude capacities, allocator overhead,
immutable definitions, observations and external snapshots.

Full Standard and broader native RL/search acceptance remain incomplete.
