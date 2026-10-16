# Resolution-time token quantities and simultaneous batches

[Issue #98](https://github.com/altf4/rustmage/issues/98) adds reusable scaled token
creation. Coverage is 998/5,164 pinned Standard identities (4,166 unsupported),
six regression definitions, 48 token factories and one emblem factory. The
[admission record](scaled-token-admissions.json) contains all eleven complete
Oracle bodies and pinned XMage source hashes at revision
492369c7844b8d76479f0393ff9eb6b0b0cf30d7.

`Instruction::CreateTokensScaled` uses immutable `AmountValue` expressions.
Existing `StatValue`/`CountFilter` queries provide exact battlefield/graveyard
counts, subtype membership, kind unions and non-token filters. Controller life
is a shared stat value; a separate amount expression reads total positive life
gained this turn. The quantity is evaluated once when the instruction executes,
before newcomers exist. Negative quantities produce zero tokens. Later gains,
zone changes or controller counts affect a still-pending effect when it resolves.
This follows [CR 608.2h](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).

`PlayerView.life_gained_this_turn` is exact u128 state, separate from current life
and the u64 number of gain events. Losses and life payments don't subtract from it;
zero/negative gains don't increment it. Lifelink uses the existing actual event
amount, including one source damaging several recipients. It resets for both
players at the start of each turn. Counter overflow uses the existing atomic
capacity error and restores the entire action, event/stack state and revision.
Observation schema **41** exposes the total. ABI 1/control response schema 2 remain
unchanged. The in-process simulation loop uses native typed state.

| New complete body | New quantity and composed rules |
| --- | --- |
| Aatchik, Emerald Radian | Artifact/creature graveyard union counts each card once; Insect deaths grow Aatchik and drain opponents |
| Ajani, Caller of the Pride | Optional +1 counter target, flying/double strike, and life-total Cat quantity; loyalty limit/costs and departed source work normally |
| Brass's Bounty | One real Treasure per controlled land, with immediately usable intrinsic token mana ability |
| Clachan Festival | Kindred enchantment; fixed two-token entry and paid one-token activation |
| Gather the White Lotus | Plains subtype count, then an owned private scry continuation |
| Germinate Recruits | Total positive life gained; Cadet name stays distinct from Wizard/Soldier subtypes |
| Krenko, Mob Boss | Live owned Goblin count, including self, changeling and applicable Kindred permanents |
| Moogles' Valor | Freeze creature count, create lifelink Moogles, then grant indestructible to all current owned creatures |
| Morcant's Eyes | Kindred Elf card; upkeep surveil and sacrificed-source graveyard Elf count, including itself |
| Sally Pride, Lioness Leader | Nontoken creature count includes Sally; attacks put counters on owned creatures |
| The Unbeatable Squirrel Girl | Exact nonrules ability-word labels, entry/attack token and paid Squirrel-count batch |

Kindred classification follows its other card type while retaining the full kind
mask, printed type line and creature subtypes (CR 308). Both admitted Kindred
objects remain noncreature enchantments. Subtype queries now also retain the
pinned printed Bear, Goblin/Warrior and Giant metadata of the three old non-Standard
regression creatures, which had only a generic Creature fallback. Their source
hashes are recorded separately. Token characteristics/names still use the existing
complete factory compiler (CR 111); only factories referenced by fully admitted
bodies are materialized. Unsupported X mana costs, copied/attacking tokens, unknown
counts/events/abilities and residual clauses retain whole-card gates.

Token creation accepts exact u128 quantities and preflights conversion plus the
complete required CardId arena range, accounting for reusable token slots. It
never silently narrows to u16 or truncates a batch. Unrepresentable quantities and
failed requested vector reservations produce recoverable capacity errors.
All created tokens exist before entry observers inspect the batch. The post-event
observer views are frozen once in `finish_zone_events`; queuing observers causes
no intervening zone changes. This removes the repeated full-arena observer scan
for each newcomer. Event multiplicity/order, limits, existing death/LKI handling
and linked returns remain owned by the shared pipeline. Other derived-stat work
can still depend on battlefield size.

No new mutable quantity continuation is required: the amount is evaluated and the
whole batch is created within one instruction. The program counter then advances
to any following private choice. Snapshot/restore during Gather's scry preserves
created tokens and never executes creation again. Arena generations, reused slots,
free-token lists, observer ledgers and pending triggers remain complete snapshot
state. The new per-player totals add 32 initialized bytes to the same benchmark
root (19,648 versus 19,616). Static rules, spare capacity, allocator/COW metadata,
observations and external snapshots remain outside the logical payload metric.

Author validation: 648 Rust tests (636 unit and twelve integration), 114 compiler/
data tests, 79 release Python tests, catalogue consistency, rustfmt and Clippy with
warnings denied. Fifteen new native tests cover every new body, quantity timing,
self/changeling/Kindred membership, union/nontoken filters, graveyard sacrifice,
fixed/scaled batches, zero counts, life totals/events/lifelink, exact wide JSON,
u128/arena capacity rollback, private scry/surveil and complete replay. A 65,536-Cat
batch crosses the old u16 boundary without truncation. Existing observer, event
multiplicity/limits and linked-return regressions also pass after batch freezing.

Three public Python/C ABI tests independently check Germinate's gain total,
Krenko's live Goblin quantity/generations and Gather's creation plus private scry
clone/restore. The scry fixture rejects a forged graveyard destination atomically.
All source hashes match the pinned repository; the subtype repair hashes were also
checked against the pinned Git blobs.

One hundred legal 60-card Standard games complete naturally: 56,332 decisions,
7,521 independent public facts, 549 checked batches, 14,116 clone/replay boundaries
and 36,808 maximum initialized state bytes. The workload exercises token effects
from all eleven admissions. Focused fixtures cover their remaining abilities.
The smoke policy delays attacks until turn 16, avoids blocks and limits optional
repeated token growth to keep this workload bounded; those choices don't change
engine legality or reduce its returned action space. Broader Standard and native
RL/search acceptance remain outstanding.

A local unpinned release `benchmark 1000` sample produced 571,049 decisions at
394,078 decisions/second (690 games/second), clone 148 ns and restore 114 ns. This
limited workload includes smoke policy/observations and excludes model inference,
encoding, JSON and an equivalent Java workload. Earlier snapshot samples varied;
these measurements don't establish an attributable speed change or full Standard
throughput. The all-Standard library goal remains active.
