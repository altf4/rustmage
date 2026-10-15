# Equipment and attachment foundations

Equipment cast as ordinary artifacts and enter unattached. Their equip abilities
use the existing activation, target declaration, payment and stack machinery:
fixed costs, target creature you control, and sorcery timing. Flash on an Equipment
does not make its equip ability usable at instant speed. Entry attachment triggers
use their own timing. If the required target is unavailable when such a trigger
would be put on the stack, that trigger is removed while the Equipment remains.

Each mutable object has an optional `attached_to: ObjectRef`. The reference binds
one exact creature incarnation. Resolution validates both source and destination;
a departed or returned source cannot attach through its old activation. A failed
target leaves any existing attachment in place and does not refund paid costs.
Re-equipping the same creature remains legal. The link resets when the Equipment
changes zones. An invalid or departed equipped creature causes detachment through
the shared frozen state-based event machinery, including when a legend choice
suspends that event. Returning a creature never restores its old attachments.

A live Equipment-source index supplies fixed additive stat bonuses and keyword
grants. Shared effective-keyword queries feed attacks, blocking, targeting,
activation tap costs, mana funding, damage and last-known information. Keyword
changes feed earlier-layer creature filters before additive stat modifiers. No
cache needs invalidation when Equipment moves or a turn changes. Cleanup removes
temporary effects while the attached continuous bonuses remain.

Mana funding retains relevant Equipment objects and their attachment edges in its
small resource graph, recomputes granted haste as resources depart, and includes
ownership and attachment identity in its memoization key. This preserves legality
for intrinsic and activated creature mana sources without copying the entire game.
Simultaneous zone transitions now pass their frozen pre-event object views into
the actual transition, so existing stack/queued abilities retain the correct
last-known keywords even if their Equipment departs earlier in the movement batch.

Observation schema 22 exposes public object attachment references and effective
stats/keywords. Links, indexes and frozen detachments belong to snapshots;
`state_bytes()` includes their initialized storage. The smoke policy avoids repeat
equip activations during one turn using the existing usage history; this is a
policy choice and does not remove those legal actions from the engine.

The compiler accepts complete Equipment definitions with fixed stat/keyword
bonuses, ordinary fixed-mana equip costs, and wholly supported remaining text.
Source entry-attachment effects can also grant exact temporary keywords to their
target. If the Equipment departs before that ability resolves, attachment fails
but an independent grant to a still-legal target can resolve.

Twenty-two complete Standard definitions are admitted:

| Cards | Supported behavior |
| --- | --- |
| Basilisk Collar; Fireshrieker; Hard-Won Jitte; Short Bow; Swiftfoot Boots; Sword of Vengeance; Vibranium Energy Daggers | Fixed continuous bonuses and equip |
| Celestial Armor; Coral Sword; Illvoi Light Jammer; Malamet Scythe; Meltstrider's Gear; Squire's Lightblade; Stolen Stark Tech; Twin Blades | Entry attachment, flash where printed, exact temporary grants |
| Crude Bent Blade; Lion Heart; Skateboard | Existing targeted entry sacrifice/damage/tap effects |
| Glimmerlight; Gold Pan; Ragged Short Spear; Well-Worn Spatula | Existing token, discard-payment and life-gain continuations |

Coverage is 656/5,164 Standard identities, six regression cards and 35 token
factories. Auras and their targeted/non-targeted entry choices, granted triggered
or activated abilities, equipped-creature event subjects, variable/conditional
bonuses, base/type changes, timestamp-dependent layers, alternative/restricted
equip costs, reconfigure, control changes and remaining card rules still require
implementation. Residual unsupported text continues to reject the whole card.

Native tests cover paid and entry attachment, illegal targets/timing, costs and
fizzling, stale source/target incarnations, cleanup, relevant combat keywords,
keyword-filtered lords, intrinsic/activated mana funding, changing funding edges,
simultaneous departure LKI and frozen detachment/legend choices. Every admitted
definition participates in source-index lifecycle fixtures. Python covers paid
equip and flash entry attachment with clone/restore through the native library.
One hundred supported Standard games completed naturally with 1,081 attachment
changes, 1,769 paid Equipment activations and 3,063 replayed boundaries; maximum
initialized state was 11,860 bytes. This excludes spare capacity, allocator
metadata, snapshots and static definitions, and is not a full Standard performance
measurement or XMage comparison.

Rules: [Comprehensive Rules, September 25, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
CR301.5, 701.3, 702.6 and 704.5n/p.
