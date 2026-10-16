# Choose-one modal triggered abilities

Issue: https://github.com/altf4/rustmage/issues/174

Twelve complete bodies bring executable coverage to **1,315/5,164** pinned
Standard cards, with **3,849 unsupported**. Six regression identities and one
emblem remain separate. There are now **57 token factories**: this stage adds
the ordinary white 1/1 Human Soldier required by Cosmogrand Zenith. Full Standard
migration remains in progress, pinned to October 15, 2026.

| Card | Complete body and all modes |
| --- | --- |
| Apothecary Stomper | 4/4 vigilance; entry chooses two +1/+1 counters on an owned creature, or gain 4 life |
| Coliseum Behemoth | 7/7 trample; entry chooses destroy an artifact/enchantment, or draw one |
| Cosmogrand Zenith | 2/4; second owned spell each turn chooses two 1/1 white Human Soldiers, or a +1/+1 counter on each owned creature |
| Divining Duelist | 3/2 flash; entry chooses tap a creature, untap a creature, or draw then discard |
| Downwind Ambusher | 4/2 flash; entry chooses an opposing creature gets -1/-1 until end of turn, or destroy an opposing creature dealt damage this turn |
| Elven Raft-Steerer | 3/2; owned landfall chooses tap an opposing creature, or untap an owned creature |
| Ferocification | Enchantment; beginning of owned combat chooses an owned creature gets +2/+0, or gains menace and haste, until end of turn |
| Gaius van Baelsar | Legendary 3/2; entry chooses each player sacrifices a creature token, a nontoken creature, or an enchantment of their choice |
| Giant-Sized Flying Ant | 3/2 flash, flying; entry chooses tap or untap a nonland permanent |
| Glamermite | 2/2 flash, flying; entry chooses tap or untap a creature |
| Lord Skitter's Butcher | 2/3; entry chooses a 1/1 black Rat that cannot block; optional sacrifice of another creature for scry two then draw one; or menace for current owned creatures until end of turn |
| Wardens of the Cycle | 3/4; owned end step with intervening Morbid chooses gain 2 life, or draw one and lose 1 life |

`modal-trigger-admissions.json` records all twelve full Oracle bodies and **108
SHA-256 entries across 52 unique Java files** at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Constructors were read in full.
Mode/event/target classes, precise conditions, full branch effects, costs,
intrinsic abilities and both token classes are recorded. Every recorded blob
matches the pinned revision and read-only checkout.

## Declaration, targets and selected programs

Immutable trigger data can contain one definition-only `Instruction::Modal`
header holding static `SpellMode` branches. The compiler emits it as the entire
trigger program. It is never an executable instruction: the engine selects a
branch before any resolution frame starts. Existing stack `mode: Option<u16>`
and announcement state own the choice. No mode definitions, strings or program
graphs are copied into simulation state.

`SpellView::modes()` and `program()` provide one selection path for spells,
triggers, targeting, resolution and the smoke policy. When placing a modal
trigger, the controller must choose a legal mode before selecting that mode's
targets. Modes whose required targets are unavailable are filtered out; optional
zero-target or non-targeted modes remain available. If no mode is legal, the
trigger is removed. The mode cannot change after commitment, including when
its target becomes illegal. This follows
[CR700.2b–c](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).

Mode choices suspend the existing APNAP trigger-ordering flow. Multiple owned
triggers retain the player's chosen stack order; the other controller places
their batch afterward, with no priority window between placement choices.
Choosing a mode neither casts a spell nor opens a mana-payment window. It does
not increment the cast counter. The selected immutable branch and exact source
reference survive source departure, a new source incarnation, private sacrifice,
scry and discard choices, clones and restores. Source reentry creates a separate
new trigger with its own mode.

Existing target predicates are applied at declaration and resolution. A selected
mode whose sole target becomes illegal fails to resolve; another legal mode
cannot replace it. Ward can counter the selected ability while its source
creature remains on the battlefield. Temporary growth/keywords expire normally;
permanent counters remain. Existing event and intervening-if checks apply before
mode placement and again before resolution as appropriate.

## Compiler and API scope

Only exact `choose one` trigger headers with 2–64 complete bullet alternatives
compile. The existing trigger parser authenticates the event, source aliases,
controller, step and intervening condition. The existing complete modal branch
compiler validates every alternative. Unknown branches or residual clauses gate
the entire card. Untracked events, choose-one-or-both, up-to-one modes, repeated
modes, modal activations, mode costs and copy/replacement mechanics remain gated.
A card with one simple supported branch never gains execution if another branch
is unsupported.

The existing `ChooseMode`, `mode_choices`, and stack/announcement `mode` fields
serve these declarations. The historical **`spell_mode` decision label also
covers triggered mode selection**. Mode choices/actions belong to the controller;
other viewers receive no choice payload or actions. Observation schema **51**,
C ABI **1** and control protocol **2** are unchanged. There is no new mutable
state field, cache or snapshot layout.

## Validation

Twelve native tests cover both seats and **all 27 modes**, with every complete
body actually cast. They include owned second-spell/reset, landfall and step
conditions; required-target filtering and no-legal-mode removal; mode-before-
target declaration; APNAP and owned ordering; source independence and distinct
reentry; illegal/stale actions; exact target revalidation without mode fallback;
Ward; private sacrifice/scry/draw; token creation and intrinsic restrictions;
temporary effect expiry; and atomic counter/life overflow during real selected
branch resolution. Semantic choices use complete clone/restore comparisons.

Three compiler tests cover exact events, targets and all-mode metadata,
2–64 bounds, malformed bullets, composition, unsupported events/modes and full
admission residual gates. One fresh release Python method resolves all twelve
bodies with public mode-before-target/privacy assertions, independent branch
effect checks and payment/private/mode replay.

Two native Standard100 runs use three legal 60-card/four-copy deck families and
reproduce:

- **70,950 decisions**, casts `[38,36,90,104,46,65,120,67,114,114,77,102]` in table order.
- Triggers/finished triggers `[38,36,19,104,29,26,425,67,114,114,77,146]`.
- **All 27 modes chosen and independently verified**, including both damaged-target destructions from Downwind Ambusher.
- **1,195 resolved mode programs**, **143,062 public stat/ability pairs**,
  **68,417 creature-history checks**, **45 private choices** and **13,536 native
  replay boundaries**.
- **21,516 maximum initialized logical state bytes** in this workload.

The independent witness reads public facts and manually models each selected
branch, target/controller/zone identity, life, draws/discards, sacrifices,
tokens, counters and tap state. A separate exact-incarnation/turn ledger tracks
resolved temporary growth/keyword grants, including squad grants to current
recipients, for independent stat/ability checks. Mode data is not read from the
engine's private instruction definitions.

Complete checks: **1,086 Rust tests** (1,074 unit, 12 integration), **207
compiler/data tests**, **142 fresh release Python tests**, all-target Clippy
with warnings denied, formatting, catalogue consistency and diff checks.

The separate native 1,000-game benchmark completed 571,049 decisions at roughly
371,723 decisions/s and 651 games/s, clone 131 ns and restore 125 ns in this run.
Initial logical state remains **19,784 bytes**. Logical accounting excludes
capacity/allocator overhead, COW sharing, immutable definitions, observations
and external snapshots. Timing depends on the host; the benchmark includes
policy and observation construction, excludes model inference/tensors/JSON and
is not an XMage parity comparison.
