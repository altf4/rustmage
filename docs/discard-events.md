# Discard and cycling trigger events

Actual discard causes now share a simultaneous batch hook: cycling activation
costs, mandatory resolution discards, optional discard payments and cleanup.
Ordinary hand-to-graveyard moves, milling and mulligan/bottom operations do not
emit these events. Each-card observers trigger once per card; one-or-more
observers trigger once per affected player in a batch, with the actual count.
Empty or exhausted hands cannot produce fictitious discarded cards.

Shared cycling definitions carry an explicit cycling marker. Basic-land and
subtype cycling have the same marker; an ordinary discard of a cycling card is
not cycling. Battlefield observers see the discard, while the discarded card's
self-discard/cycle trigger uses that card in its destination zone. Triggered
abilities wait until the activation or resolving effect completes, then use
ordinary APNAP ordering and target declarations. Cycling's draw/search ability
is already on the stack beneath the resulting triggered abilities.

Schema 28 adds public `discard_event` context to stack/announcement/trigger views:
discarding player, captured amount and whether this was a cycling cost. The
context is owned value data, retained through private choices, source departure,
cloning and restoration. Quantity-dependent counters require the exact original
battlefield incarnation; source damage retains its existing last-known facts and
keywords. A returned source is a new object. Private selections reveal no additional card identities before the discard
batch commits; an announced cycling source is already public.

Cleanup now stores private provisional selections and moves all chosen cards
simultaneously. It then removes damage and temporary effects. Waiting triggers
cause a cleanup priority cycle, followed by another cleanup step after the stack
empties. Draws during that priority cycle can require another hand-size discard.
Snapshots and initialized-state accounting include the provisional selections.

Creature-or-Vehicle targets check the exact permanent type/subtype, relation,
hexproof and incarnation. Cycling triggers can be countered by ward independently
of the underlying cycling draw/search. Optional targets can be omitted, and a
failed target incarnation cannot modify a later object.

These rules follow CR701.9, 702.29c–f, the triggered-ability rules and CR514.1–3 in
the [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).
The pinned Java cycling/discard trigger implementations distinguish the same
causes; this engine stores compact event facts instead of Java watcher objects.

Eight complete Standard definitions are added:

| Card | Complete admitted semantics |
| --- | --- |
| Agonasaur Rex | Trample, fixed cycling, optional creature/Vehicle target counters plus temporary trample/indestructible |
| Hobgoblin, Mantled Marauder | Flying/haste; source growth for each discarded card |
| Howler's Heavy | Fixed cycling; opponent creature/Vehicle temporary debuff |
| Magmakin Artillerist | Captured discard-count damage, fixed cycling and self-cycle damage |
| Marauding Mako | Discard-count source counters and fixed cycling |
| Scrounging Skyray | Flying, discard-count source counters and fixed cycling |
| Tinybones, Pocket Nuisance | Enter opponent discard; separate one-or-more observer per discarding player |
| Titanbones, Towering Heart | Reach, life-gain source counters and self-discard life gain |

Validation includes 395 Rust tests (387 unit and eight integration), 70 compiler
tests, 43 rebuilt release Python tests, catalogue consistency, rustfmt and Clippy
with warnings denied. Fifteen native event fixtures cover costs/effects/cleanup,
private simultaneous selections, APNAP, quantities, exhausted hands, source LKI,
exact object identity, ward, conditional target legality, extra cleanup and
branch replay. A metadata-only unsupported Vehicle is used solely to verify the
noncreature target predicate; it remains rejected from submitted decks. Direct
cause-hook/LKI fixtures are distinguished from printed-card paths.

One hundred supported Standard games completed naturally: 48,493 decisions,
1,088 event-bound announcements, 900 cycling-bound announcements, 31 multi-card
announcements, 14 cleanup discard choices and 3,610 cloned/replayed boundaries.
Maximum initialized state was 12,980 bytes, excluding allocator metadata, spare
capacity, snapshots and static definitions. The smoke policy uses the actor's
observation. This is a rules/replay workload, not an XMage speed comparison.

Coverage is 767/5,164 Standard identities, six regression cards and 39 token
factories. Discarded-card reference effects, conditional/reflexive triggers,
mode-history tracking, variable cycling costs, broader card mechanics and native
RL/search integration remain incomplete. Full Standard acceptance is outstanding.
