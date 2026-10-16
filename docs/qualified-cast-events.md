# Qualified completed-cast events

Issue: https://github.com/altf4/rustmage/issues/190

After the source-tap stage, thirteen complete bodies bring executable coverage
to **1,384/5,164** pinned Standard identities, with **3,780 unsupported**. Six regression identities,
57 token factories and one emblem remain separate. Full Standard migration
remains incomplete; the target is pinned to October 15, 2026.

| Card | Entire supported body |
| --- | --- |
| Angry Rabble | Trample; owned MV≥4 cast deals one to opponents; sorcery-only `{5}{R}` puts two counters on this source |
| Brineborn Cutthroat | Flash; owned cast during an opponent's turn puts a source counter |
| Enraged Flamecaster | Reach; owned MV≥4 cast deals two to opponents |
| Kulrath Mystic | Owned MV≥4 cast grants source +2/+0 and vigilance until end of turn |
| Lurking Lizards | Trample; owned MV≥4 cast puts a source counter |
| Mage Tower Referee | Artifact creature; owned multicolored cast puts a source counter |
| Nightmare Sower | Flying, lifelink; owned cast during an opponent's turn puts a −1/−1 counter on up to one creature |
| Skybeast Tracker | Reach; owned MV≥5 cast creates a complete Food |
| Solarium Sentry | Opposing MV≤2 cast gains two life |
| Spider Manifestation | Reach; tap for red or green; owned MV≥4 cast untaps this source |
| Stormkeld Prowler | Owned MV≥5 cast puts two source counters |
| Tanufel Rimespeaker | Owned MV≥4 cast draws a card |
| Unwelcome Sprite | Flying; owned cast during an opponent's turn surveils two |

`qualified-cast-admissions.json` records the full Oracle bodies and **100 hash
entries across 48 unique Java files** at pinned revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All thirteen constructors were read;
the provenance includes spell-event dispatch, MV/color filters, turn qualification,
effects, intrinsic abilities, mana abilities and Food's full recipe.

## Immutable filters and event timing

Existing `CastFilter::ManaValueMinimum` is reused. New maximum and multicolored
filters use the same printed characteristics as the other supported spell rules.
Mana value is distinct from mana actually spent. A two-brid spell with printed
value six still matches the five-or-greater filter after paying three mana.
Hybrid symbols can make a spell multicolored even when paid with a single color.
Control/color/type-changing mechanics remain gated.

`TriggerCondition::DuringOpponentsTurn` is immutable event metadata. Registry
classification unwraps it like the existing limit and intervening-if wrappers.
The completed-cast collector checks the active player while collecting observers;
resolution does not recheck this event-only qualification. Nested intervening-if
and once-per-turn metadata preserve their existing behavior. The synthetic wrapper
unit fixture exercises those metadata combinations without admitting a partial card.

Proposed announcements, targets and mana payments do not trigger anything until
casting completes. A spell later countered still caused its cast event. Activations
are not casts. The source index, captured source incarnation, APNAP ordering,
private choices and rollback are reused. Untapping Spider Manifestation is a
stack trigger; it cannot supply hypothetical future mana to fund its triggering
spell. Leaving and reentering a source cannot redirect an old source-counter or
untap effect. A targeted Nightmare ability revalidates its selected incarnation;
choosing zero targets remains legal.

The compiler admits only exact closed MV headers, the multicolored header and
the opponent-turn header. Bounds above 65,535, unknown qualifications, extra
restrictions, unsupported spell subjects and unknown residual bodies stay gated.
Helga, Superior Foes, Rhino, Dream Spoilers and Voracious Tome-Skimmer retain
remaining unsupported body gates. No proposed announcement is approximated as a cast.

There are **no new mutable fields or caches**. Observation schema **52**, C ABI
**1**, control protocol **2**, all existing executable definitions and card/token
identities are unchanged. Initial logical state remains **19,848 bytes**.

## Validation and independent Standard witness

Fourteen native tests cover complete casts for both seats, printed MV boundaries,
relations, multicolored hybrid symbols, actual cheap two-brid payment, event-only
turn capture, announcement and payment boundaries, a real counterspell, optional
and stale targets, source departure/reentry, private surveil, atomic counter/cast
count overflow, Angry's full activation, both Spider mana routes, pure funding and
Food's full activation. Semantic actions compare full snapshots, clones and replay,
including stale action-index rejection. Two compiler tests cover complete admission
and closed headers/wrappers/residual gates. One release Python method runs thirteen
whole-card scenarios with independent public effects and private-choice replay.

The Standard workload uses two legal 60-card/four-copy families and 24 basic lands.
One family includes Defibrillating Current and chooses affordable cheap two-brid
payments; the other includes Opt and Lightning Helix. Opt is held for opposing
turns to exercise the three event-only observers.

The witness parses printed mana symbols independently rather than calling the
engine's MV/color matchers. It reconstructs expected triggers for every completed
spell using public source/controller/active-player facts, then reconciles missing
and unexpected packets by source incarnation and ability index. A separate ledger
models resolved permanent counters, temporary power/vigilance, life, damage, Food,
draws and untaps. Exact surveil destinations and library counts are checked apart
from Opt's separate scry-then-draw continuation. Public stat/keyword witnesses
cover every battlefield creature in both observations. Every action is cloned,
restored and replayed; the witness reads no private trigger definitions/programs.

Two release Standard100 runs reproduce:

- **57,533 decisions**; casts `[68,101,78,95,70,101,84,86,72,109,104,89,122]`;
- triggers `[16,11,19,63,74,31,33,29,87,59,8,71,17]`;
- verified effects `[14,11,19,63,73,31,33,29,87,59,8,71,17]`;
- **4,164 independent cast/source qualification checks**, **174,230 public
  stat/ability pairs**, **120 private choices**, **57,533 replay boundaries**;
- **21,664 maximum initialized logical bytes**.

Terminal-producing programs are excluded from verified-effect counts. Native
full-body tests separately exercise the supplemental activated abilities.

Before integration, standalone validation passed **1,191 Rust tests**
(1,179 unit and 12 integration), **227 compiler/data tests**,
**150 fresh release Python tests**, catalogue
consistency, formatting, all-target Clippy with warnings denied, diff checks
and unchanged-definition checks. The unpinned 1,000-game smoke benchmark measured 571,049 decisions at
approximately 96,863 decisions/s and 170 games/s, clone 255 ns and restore 244 ns
while other validation jobs shared the host. These noisy measurements are not
performance parity claims. The benchmark includes smoke policy and observations,
excludes model inference/tensors/JSON, and is not an XMage comparison. Logical
bytes exclude spare capacity, allocator overhead, immutable definitions,
observations and external snapshots.

Standalone catalogue fingerprint:
`60eb08f53e1629a2700b497ef2bac07def952ca2648aca464841f06f40678eb7`.

The final branch incorporates the independently reviewed source-tap stage.
Combined validation passed **1,211 Rust tests** (1,199 unit and 12 integration),
**230 compiler/data tests**, **151 fresh release Python tests**, all-target
Clippy, formatting and catalogue consistency. The qualified-cast Standard100
reproduced every vector above; the source-tap Standard100 separately reproduced
70,443 decisions and 57,094 replays with all thirteen tap effects. Provenance
was reverified against pinned blobs and checkout. Exactly thirteen previously
unsupported immutable definitions change relative to the source-tap base;
all earlier executable definitions remain byte-identical.

Final catalogue fingerprint:
`43304f0d919fe89ac4d50a4a699fd02ecb9f8d88b2b920ea1ec030b5c279a1d2`.
