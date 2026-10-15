# Player-damage triggers and captured amounts

The engine dispatches player-damage events after applying the whole damage batch,
before state-based actions. Zero and negative amounts do not trigger. Combat-only
conditions exclude other damage, opponent-only conditions compare the recipient
to the observer's controller, and life loss does not become damage. Assignment
fragments from one source to one player in a simultaneous batch form one
occurrence. Ordinary creature observers trigger per dealer; “one or more”
observers aggregate qualifying dealers separately for each damaged player.

Observers can refer to their source, an attached creature, or controlled creatures
matching existing type/keyword/token filters and explicit counter/modified
qualifiers. A creature is modified by any supported counter, any Equipment, or an
Aura controlled by its controller. A foreign Aura does not qualify by itself.
Equipment hit triggers belong to the Equipment's controller. Granted abilities
remain distinct and unimplemented; the compiler rejects residual granted text.

An owned damage context records the exact dealer reference (or no single dealer
for an aggregate), damaged player, amount and combat flag. Amount-dependent draws
use this event, not later power or life totals. Source damage facts capture card
identity, battlefield origin, counters, modification and attacking status, with
last-known updates before zone changes. Other observers can therefore recognize
a departed battlefield source performing noncombat damage. Cards acting from
hand or as spells do not become controlled creatures. The departed source's own
battlefield ability does not re-trigger from a later incarnation.

Schema 25 adds `SpellView.damage_event`, `source_facts`, damage-source facts and
public `ObjectView.modified`/`attacking`. All are owned scalar state. Existing
APNAP trigger ordering, legal target selection and continuation snapshots apply.
Initialized vector accounting includes the live observer index; inline context
storage is included through the containing state/stack structs.

Dragon Mage's whole-hand discards use private APNAP choices, one simultaneous
commit and then both draw sequences. Selected cards stay in hand until every
player finishes choosing. Discard counts use usize rather than truncating hand
size to a fixed card-count field. Large damage-sized draws attempt at most the
remaining library plus one empty draw under currently admitted semantics. Draw
replacement and draw-event cards remain gated and will require extending this
path. Non-targeted creature tapping and exact-source sacrifice reuse ordinary
resolution and identity validation.

These rules follow CR113.7a, 603.2/603.3, 608.2h and 700.9 in the
[September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).
Fear of Failed Tests, Dragon Mage, Goldvein Pick, Haliya, SP//dr and The Thing
were also reconciled with their pinned Java source. This is source inspection,
not a differential-engine comparison.

Twenty-two complete Standard definitions are added:

| Cards | Complete newly executable behavior |
| --- | --- |
| April, Reporter of the Weird | Combat hit draws its captured damage, then discards one |
| Fear of Failed Tests | Enchantment creature, combat hit draws captured damage |
| Dragon Mage | Flying, combat hit discards all hands then draws seven each |
| Eager Trufflesnout; Scream Puff | Trample/deathtouch respectively, combat hit creates Food |
| Festerleech | Combat hit mills two, paid once-per-turn growth |
| Frostbite Pyromental | Trample/haste, combat hit draws two, end-step self-sacrifice |
| Goldvein Pick | Fixed Equipment bonus, hit Treasure creation and paid equip |
| Haliya, Ascendant Cadet | Entry/attack counters and grouped plus-counter creature hit draws |
| Invasion Tactics | Entry team growth, grouped Ally hit draws |
| Jeskai Shrinekeeper | Flying/haste, combat hit gains one life and draws one |
| Locke Cole | Deathtouch/lifelink, combat hit draws then discards |
| SP//dr, Piloted by Peni | Vigilance, entry target counter, modified-creature hit draws |
| The Thing, Ben Grimm | Trample, grouped Hero damage to players grows the source |
| Thieving Otter | Any damage to an opponent draws one |
| Ultimecia, Temporal Threat | Entry taps opponents' creatures, own creature hit draws |
| Yathan Tombguard | Menace, counter-bearing creature hit draws and loses life |
| Appendage Amalgam; Fear of Surveillance | Enchantment creatures, flash/vigilance respectively, attack surveil |
| Grasping Longneck | Enchantment creature, reach, death gains two life |
| Ball Lightning | Trample/haste and exact-source end-step sacrifice |
| Syr Vondam, the Lucent | Deathtouch/lifelink, entry/attack grants other creatures growth/deathtouch |

Validation includes 346 Rust tests (338 unit and eight integration), 61 compiler
tests, 37 release Python tests, catalogue consistency, rustfmt and Clippy with
warnings denied. Fixtures cover combat/trample/double strike, individual versus
aggregate multiplicity, Equipment controller, counter/modified/tribal filtering,
source departure and replay, hidden discard choices, bounded empty draws,
non-targeted entry tapping and exact-source sacrifice. Synthetic IR explicitly
covers a departed Hero dealing noncombat damage; no partial SP//dr damage ability
is admitted. Synthetic multi-recipient batches check per-player grouping.

One hundred supported Standard games completed naturally: 58,554 decisions,
773 observed trigger announcements, 71 aggregate announcements, 266 Equipment
announcements, 233 event-bound discard choices and 1,006 cloned/replayed
boundaries. Maximum initialized state was 12,564 bytes for this workload,
excluding allocator metadata, spare capacity, snapshots and static definitions.
These are smoke workloads, not competitive decks or an XMage speed comparison.

Coverage is 712/5,164 Standard identities, plus six regression cards and 36 token
factories. Granted and temporary triggers, additional damage subjects, prevention
and replacements, wider event bindings, variable effects, control/type layers and
remaining card mechanics are still incomplete. Full Standard acceptance and
remaining native RL/search integration remain outstanding.
