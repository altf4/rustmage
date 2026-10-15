# Successful draw events and per-turn ordinals

Every successful individual draw increments that player's owned per-turn count
and captures a drawing-player/ordinal event. Multi-card instructions produce one
event per card; instructions affecting both players draw in APNAP order. Counts
reset for both players at each new turn, including draws outside their own turn.
An observer entering after the first draw still sees the second draw correctly;
entering after the second draw cannot retroactively trigger. Opening-hand and
mulligan counts are reset before the first turn, whose normal draw is skipped.

Failed draws retain the normal failed-draw loss flag, but increment no successful
draw count and trigger no successful-draw observer. Qualified library searches,
recovery and ordinary library-to-hand moves are not draws. Cycling, Clues,
upkeep/draw-step actions and spell/program draws share the same hook.

Battlefield observer indices avoid scanning every object during each draw.
Triggers wait until the resolving program completes, including private rummage
choices, and then use normal APNAP ordering and target declarations. Schema 29
exposes `cards_drawn_this_turn` for each player and `draw_event` on stack,
announcement and pending-trigger views. Captured facts contain only the drawing
player and ordinal; they reveal no drawn card identity. Values, source identities,
private continuations and counts remain part of complete snapshots and replay.

Exact draw-event bodies can refer to the drawing player independently of the
source controller or later draws. Source counters still require their original
battlefield incarnation. Targeted counters/temporary keywords revalidate their
targets, can trigger ward, and keep permanent counters after temporary keywords
expire. Two-target-player draw activations require two distinct legal targets.
Tiger-Seal's upkeep tap and second-draw untap use ordinary stun replacement;
Otter-Penguin's growth and unblockability expire during cleanup.

These rules follow CR121.1–5, the triggered-ability rules and turn structure in
the [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).
The pinned Java DrawNthCardTriggeredAbility and draw watcher track the same
successful-draw ordinals. Rust stores counts directly in each player state.

Seventeen complete Standard definitions are added:

| Cards | Complete admitted semantics |
| --- | --- |
| Atlantean Cavalry; Erudite Wizard; Lakeshore Apothecary | Second-draw source counters and printed vigilance where present |
| Clinquant Skymage; Ravenhill Flock | Flying and source counters for each draw |
| Bard the Bowman | Reach; second-draw targeted counter plus temporary lifelink |
| Gleaming Splendor | Opponent second-draw Treasure; two-target-player draw activation |
| Knowledge Seeker | Vigilance, second-draw counters and death Clue |
| Loxodon Eavesdropper | Enter Clue; second-draw temporary growth/vigilance |
| Mischievous Mystic | Flying; second-draw flying Faerie token |
| Otter-Penguin | Second-draw temporary growth/unblockability |
| Roxxon Brutes | Menace, targeted second-draw counter and basic landcycling |
| Scrawling Crawler | Upkeep draws for both players; opponent-draw life loss bound to that player |
| Tiger-Seal | Vigilance, upkeep self-tap and second-draw untap |
| Lightfoot Technique; Maximum Overdrive; Take Up the Shield | Targeted permanent counter and exact temporary keyword pairs |

Validation includes 415 Rust tests (407 unit and eight integration), 73 compiler
tests, 45 rebuilt release Python tests, catalogue consistency, rustfmt and Clippy
with warnings denied. Twenty native fixtures cover individual quantities,
opponent-turn resets, late entry, source departure, draw failures, cycling/Clues,
non-draw hand movement, upkeep/draw timing, APNAP, privacy, private rummage,
target legality/ward, temporary effects, mulligans and branch replay. Direct
source/draw scheduling fixtures are distinguished from complete printed-card
cast, activation, search, cycling and upkeep paths.

One hundred supported Standard games completed naturally: 50,550 decisions,
2,501 successful in-game draws, 1,518 event-bound trigger announcements, 961
ordinal-two announcements and 4,871 cloned/replayed boundaries. Maximum
initialized state was 13,872 bytes, excluding allocator metadata, spare capacity,
snapshots and static definitions. The smoke policy uses only the actor's
observation. This is a rules/replay workload, not an XMage speed comparison.

Coverage is 784/5,164 Standard identities, six regression cards and 40 token
factories. Draw replacement/prevention, conditional/reflexive/limited triggers,
copy effects, ability-removal layers, remaining card semantics and native
RL/search integration remain incomplete. Full Standard acceptance is outstanding.
