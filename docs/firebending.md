# Firebending and combat-retained red mana

Issue: https://github.com/altf4/rustmage/issues/164

Eight complete bodies bring executable coverage to **1,254/5,164**, with **3,910
unsupported**. Six regression identities, 56 token factories and one emblem
remain separate. Standard stays pinned to October 15, 2026.

## Complete bodies and provenance

| Card | Whole body |
| --- | --- |
| Azula, On the Hunt | Legendary 4/3; Firebending 2; independent attack trigger loses one life and creates a complete Clue |
| Fire Sages | 2/2; Firebending 1; `{1}{R}{R}` puts a +1/+1 counter on itself |
| Namazu Trader | 3/4; entry loses one life and creates a complete Treasure; attack may sacrifice another creature or artifact to surveil 2 |
| Rough Rhino Cavalry | 5/5; Firebending 2; Exhaust `{8}` puts two +1/+1 counters on itself and gives it trample until end of turn |
| Sun Warriors | 3/5; Firebending X counts current owned creatures on resolution; `{5}` creates a 1/1 white Ally |
| Uncle Iroh | Legendary 4/2 with two red/green hybrid pips; Firebending 1; owned Lesson spells cost one generic less |
| Vindictive Warden | 2/3 with a black/red hybrid pip and menace; Firebending 1; `{3}` deals one damage to each opponent |
| Zhao, Ruthless Admiral | Legendary 3/4 with two black/red hybrid pips; Firebending 2; sacrificing another owned permanent gives current owned creatures +1/+0 until end of turn |

`firebending-admissions.json` records all eight Oracle bodies and 80 authenticated
SHA-256 entries across 44 unique Java files at XMage revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Constructors, shared Firebending and
attack abilities, mana-pool duration handling, effects, activation costs, Exhaust,
Lesson reduction and token recipes are included. Each pinned Git blob was
compared against the read-only checkout. No token factory was added. Namazu has
no Firebending; it is admitted by the exact life-loss plus predefined-token
composition shared with Azula.

## Mana ownership, costs and expiry

Each player stores a checked `u32` combat-retained red subset. Public
`players[].mana[3]` remains the inclusive red total;
`players[].combat_red_mana` identifies how much survives a combat-step boundary.
It is not another spendable pool to add to the total. Every receipt, payment,
clone and restore preserves `combat_red_mana <= mana[3]`.

Fixed Firebending adds red on resolution of an ordinary attack trigger. It is
not a mana ability and cannot fund an announcement while its trigger is still
waiting on the stack. Multiple instances produce independent triggers. Sun
Warriors counts owned creatures when its trigger resolves, including creatures
that arrived after attacking; the source need not remain on the battlefield.
The stored controller owns the mana and the count.

Ordinary mana empties at each step boundary. Retained red survives blockers,
first-strike damage, regular combat damage and the complete end-of-combat
priority window. It expires on entry to the second main phase. Newly produced
ordinary mana during combat still expires normally. Source departure does not
remove mana already received. These rules follow CR702.189 and the
[official Avatar release notes](https://magic.wizards.com/en/news/feature/avatar-the-last-airbender-release-notes),
with the [September 25, 2026 comprehensive rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt)
as the rules reference.

All real payments share one accounting helper: direct spells, prepaid
multitarget spells with or without available targets, staged spell payments,
activated costs (including mana conversion), and resolution-owned Ward mana.
The existing six-color payment vector spends ordinary red before retained red.
The admitted subset has no effects that distinguish the origin of spent mana;
this canonical choice preserves the longer-lived resource without branching on
otherwise interchangeable red. Virtual funding uses the inclusive pool and
remains pure. Additional costs, source-incarnation checks, target legality and
Ward ownership keep their existing enforcement. Capacity failures restore the
whole current action, including a suspended Firebending trigger.

The compiler accepts exact fixed Firebending and the owned-creature-count
variant. Source-power/last-known power, granted/copied Firebending, flashback,
firebend-event observers, Vehicles, mana-spending provenance/restrictions and
extra-combat effects remain gated with their whole bodies. Lesson reduction
matches printed Lesson subtype and owned casts; changeling grants creature
types and does not grant Lesson. Colored and hybrid requirements remain intact.

## Observation and memory

Observation schema is **51** for the public retained-mana field. Current Rust,
example and Python assertions are updated. C ABI 1 and control protocol 2 remain
unchanged. Native snapshots own the field through player state; they are not
serialized schema-50/51 files. No allocation or event payload is introduced by a
mana receipt or payment. Logical byte accounting includes the player-state
layout, excluding allocators, capacity, COW sharing, immutable definitions,
observations and external snapshots.

## Validation and Standard workload

Sixteen native tests cover both seats and all eight actual casts, exact bodies,
hybrid colors, token recipes and source-independent attack triggers. They check
all combat steps, first strike, ordinary-mana expiry, end-combat spending,
fixed/scaled receipts, changed creature counts, direct/staged/prepaid payments,
virtual funding and actual Crystal Grotto conversion, resolution-owned Ward
payment/decline, Clue draws, Exhaust lifetime/reentry, Warden damage, Zhao's
frozen team growth, Iroh's precise Lesson discount, Namazu's optional sacrifice
and private surveil, and atomic overflow. Owned decisions compare clone and
restore replay and reject stale/illegal actions.

Three compiler tests cover exact clauses, duplicate triggers, boundaries,
negative scopes and whole-body residual rejection. Two fresh release Python
tests run eight body scenarios and replay the native attack/entry frames,
public retained red, ordinary expiry and combat-end expiry.

The native `standard_firebending` workload uses two 60-card Standard deck
families with four copies per nonbasic identity. Two 100-game runs reproduce:

- 52,945 decisions; casts `[50,185,63,143,60,80,98,85]` in the table's order.
- Triggers and finished triggers both `[52,241,149,215,70,70,165,145]`.
  These include entry/attack/sacrifice triggers, not only Firebending.
- Creature activations `[0,105,0,30,28,0,93,0]`; separate Azula attack,
  Namazu attack and Zhao sacrifice triggers `[26,86,42]`; six discounted Lessons.
- Retained red received/spent/expired/held at game end `1411/667/621/123`,
  with an exact conservation assertion. Games may end during combat.
- 52,945 public subset/hand-privacy checks; 51,373 creature-history checks;
  324 private library choices; 13,040 native replay boundaries.
- 21,320 maximum initialized logical bytes in this workload.

Complete checks passed: **1,014 Rust tests** (1,002 unit, 12 integration),
**191 compiler/data tests**, **137 fresh release Python tests**, all-target
Clippy with warnings denied, formatting, catalogue consistency and diff checks.

The separate 1,000-game native Freeform benchmark completed 571,049 decisions at
roughly 411,914 decisions/s and 721 games/s. Clone averaged 415 ns and restore
425 ns in this run. Initial logical state remains **19,784 bytes**; the new
player field fits existing alignment. Timing is host-dependent. The benchmark
includes policy and observation construction, excludes neural inference,
tensors and JSON, and is not an XMage parity comparison.
