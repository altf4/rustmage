# Public-state source predicates and conditional block restrictions

Issue: https://github.com/altf4/rustmage/issues/170

Thirteen complete bodies bring executable coverage to **1,285/5,164** pinned
Standard cards, with **3,879 unsupported**. Six regression identities, 56 token
factories and one emblem remain separate. The format remains pinned to October
15, 2026; this is an incremental migration.

| Card | Complete body |
| --- | --- |
| Akawalli, the Seething Tower | Legendary 3/3; Descend 4 gives +2/+2 and trample; Descend 8 adds another +2/+2 and a one-blocker maximum |
| Cephalid Inkmage | 2/2; entry surveils three; Threshold makes it unblockable |
| Druid of the Spade | 2/3; +2/+0 and trample while its controller controls a token |
| First-Time Flyer | 1/2 flying; +1/+1 with a Lesson card in its controller's graveyard |
| Foggy Swamp Hunters | 3/4; lifelink and menace after its controller has drawn two cards this turn |
| Ghitu Lavarunner | 1/2; +1/+0 and haste with two instant and/or sorcery cards in its controller's graveyard |
| June, Bounty Hunter | Legendary 2/2; unblockable after two owned draws; `{1}`, sacrifice another creature creates a Clue, only during its controller's turn |
| Kitesail Corsair | 2/1; flying while attacking |
| Messenger Hawk | Hybrid 1/2 flying; entry creates a Clue; +2/+0 after two owned draws |
| Nightwhorl Hermit | 1/4 vigilance; Threshold gives +1/+0 and unblockability |
| Skyward Spider | Double hybrid 2/2; ward `{2}`; flying while modified |
| Stoic Sphinx | 5/3 flash, flying; hexproof while its controller hasn't cast a spell this turn |
| Twinblade Paladin | 3/3; each owned life-gain event adds a +1/+1 counter; double strike at 25 or more life |

`source-state-predicate-admissions.json` records all Oracle bodies and 129
SHA-256 entries across 59 unique Java files at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All thirteen constructors were read
in full. The manifest also includes precise conditions and effects, draw-count
helpers, modified predicates, private surveil, Clue generation, activated costs,
ward, hybrid bodies and life-gain counters. Recorded blobs match the read-only
checkout and pinned Git revision. No token definition was added.

## Rules and ownership

New immutable expressions inspect committed source attack membership, source
modification, current-turn owned draws, zero owned spells cast and controlled
tokens. Existing count and life expressions supply exact Lesson, instant-or-
sorcery and life thresholds. They use current objects and existing owned turn
histories, with no mutable cache, state field or observation change.

Attacker selection is tentative until `FinishAttackers`. Corsair's conditional
flying starts at commitment and clears after combat or source departure. Source
identity includes generation. Draw counts include actual normal/effect draws;
surveil and search do not count as draws. Both players' turn counters reset at
the turn boundary. Sphinx's own cast counts, so its conditional hexproof is off
during that turn; printed flash remains intrinsic and works from hand.

Modified uses a shared helper for both the existing public modified fact and
the new predicate: any supported counter, any Equipment attached to the exact
source, or an attached Aura controlled by its controller. Foreign Equipment
qualifies; a foreign Aura does not. Temporary growth and exhausted ability bits
are not modifications. Departed attachments and old source incarnations do not
qualify. Control-changing bodies remain gated; for currently admitted bodies,
owner and controller coincide. This follows
[CR700.9](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).

The compiler accepts exact source-only restriction sentences for unblockability
and a maximum of one blocker, using existing engine rule flags. These flags
represent restrictions, not new formal keyword abilities. Complete compound
stat/ability/restriction clauses retain one condition, and “an additional” fixed
bonus is a separate additive modifier. Akawalli is 3/3 below four permanent
cards, 5/5 at four through seven, and 7/7 at eight or more. It only gains the
one-blocker maximum at the last threshold.

Precise legendary aliases are recognized as source subjects after a leading
condition. Recognized life-gain and own-turn qualifiers remain conjunctive.
Unknown residual clauses reject the entire body. Conditional flash, persistent
“hasn't dealt damage” history, crime, entry conditions, general type changes and
unsupported ability dependencies remain gated. Ruric Thar, Magecrusher and
Karakyk Guardian are not admitted using the insufficient per-turn damage bit.

## Funding and rollback

Concrete queries and mana-funding search call the same evaluator. Funding
borrows the original game and overlays its selectively copied resource objects.
Virtual counter removal, attachment departure and token sacrifice therefore
change predicates just as concrete mutations do. Virtual actor life controls
life thresholds. Current-turn draw/cast histories are read-only during pure
mana search. Existing memo keys and resources remain sufficient for the admitted
pure mana transitions; no hypothetical unresolved trigger produces spendable
mana.

Observation schema **51**, C ABI **1** and control protocol **2** are unchanged.
Opaque snapshots already own all relevant objects, counters, attachments, combat
and histories. Initial logical state remains **19,784 bytes**. Logical accounting
excludes capacity, allocator overhead, COW sharing, immutable definitions,
observations and external snapshots.

## Validation

Thirteen new native rules/body tests exercise both seats, all actual casts,
printed abilities, hybrid payment and ward, private surveil and Clue creation.
They cover exact threshold crossings, actual block rejection, attack commitment
and cleanup, counter/Equipment/Aura/owner/incarnation distinctions, live draw
and spell history resets, Lesson and instant/sorcery graveyard counts, token
zones, source hexproof targeting, own-turn sacrifice costs, Clue draw activation,
life-gain counter triggers and real double-strike combat. Invalid actions,
stale revisions and clone/restore are checked at semantic boundaries. A real
Paladin counter-trigger overflow is atomic across state and revision.

Two funding tests compare source predicates under counterfactual counters,
attachments, life and attack commitment, and a real versus virtual Treasure
sacrifice. The original game remains unchanged. Three compiler tests cover
exact predicates, aliases, bounds, restriction composition, qualifiers, full
admissions and negative residuals. The scaling-source inventory checks 31
sources. One fresh release Python method runs thirteen real-game cases to the
condition being enabled, independently checks public values and replays private
and payment boundaries.

The native Standard100 workload uses three legal 60-card deck families with
four copies per identity. It independently reconstructs conditions, keyword/rule
flags, stacked bonuses, counters and attached Goldvein Pick bonuses from public
facts. All thirteen bodies are cast and all thirteen conditions become true.
Its successful run reports:

- 67,087 decisions; casts `[91,61,88,107,33,157,58,58,166,144,128,67,139]` in table order.
- Entry/life triggers and finished triggers `[0,61,0,0,0,0,0,0,166,0,0,0,140]`; 74 June activations.
- Enabled observations `[6710,4951,2730,17907,547,12001,1995,458,4406,8697,7310,4635,5875]`.
- 217,925 independent public stat/ability pairs, 65,136 creature-history checks,
  382 private choices and 14,438 native replay boundaries.
- 21,780 maximum initialized logical state bytes in this workload.

Full checks passed: **1,058 Rust tests** (1,046 unit, 12 integration), **201
compiler/data tests**, **140 fresh release Python tests**, all-target Clippy
with warnings denied, formatting, catalogue consistency and diff checks.

The separate native 1,000-game benchmark completed 571,049 decisions at roughly
388,998 decisions/s and 681 games/s, clone 137 ns and restore 130 ns in this run.
Timing is host-dependent; it includes policy and observation construction,
excludes neural inference/tensors/JSON and is not an XMage parity comparison.
