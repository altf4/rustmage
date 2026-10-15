# Connive continuations and sequential choices

Connive draws, privately discards, then puts counters equal to the actual nonland
cards discarded on the original permanent. Fixed connive N uses the same path.
Connive 0 produces no operation. Failed draws and exhausted hands do as much as
possible, with state-based losses deferred until the complete resolution ends.
Draw and discard observers wait until all connivers and later instructions have
completed; their captured facts remain independent.

An owned boxed continuation records the amount, remaining exact object/controller
pairs and the current conniver. Multiple permanents connive sequentially: the
first controller in APNAP order with remaining permanents chooses one, completes
its draw/discard/counters, then the sequence continues. `ChooseConniver` selects
an exact recorded object. Source departure still uses its last controller and
permanent identity to draw/discard; a later incarnation receives no counters.
Targeted effects use the conniving permanent's controller, not necessarily the
spell/ability controller. Illegal targets are removed before resolution.

Schema 30 adds public `connive_choice` and the `connive_order` decision. The view
contains public amounts and object/controller pairs, never hand selections.
Existing actor-only discard actions and private selected-card views handle each
operation. Snapshots include the whole sequence, provisional discards, draw
history, trigger queues and RNG. Initialized-state accounting includes the box
and remaining object records, excluding spare capacity.

Exact source pronouns and printed names bind only in their supported trigger
contexts. Explicit named attack prefixes, other-owned-subtype entry observers,
optional graveyard-exile targets and independent target groups reuse typed
programs. A target player drawing and another creature conniving remain separate
groups, so an absent optional creature cannot suppress the independent draw.
An optional graveyard target can be omitted without suppressing a following
life gain/draw; choosing the sole target and then losing it causes the usual
whole-effect fizzle.

These rules follow CR701.50 and targeting/trigger rules in the
[September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).
The pinned Java ConniveSourceEffect also uses permanent LKI and counts nonland
cards; Rust stores the suspended operation as owned data.

Sixteen complete Standard definitions are added:

| Cards | Complete admitted semantics |
| --- | --- |
| A.I.M. Scientists | Enter connive and basic landcycling |
| Doc Ock's Henchmen | Flash and attack connive |
| Kang, Temporal Tyrant | Named attack connive and second-draw life drain |
| Madame Masque | Enter connive and second-draw menace Villain token |
| Mechanical Mobster | Independent optional graveyard exile and owned creature connive targets |
| Prowler, Clawed Thief | Menace; source connive when another owned Villain enters |
| Red Room Recruit | Enter connive |
| Unstable Experiment | Player draw and independent optional owned-creature connive |
| Afterthought Sentry; Ambush Wolf | Flying activation/attack optional exile; flash/enter optional exile |
| Ascendant Dustspeaker | Flying, enter other-creature counter and combat-step optional exile |
| Heritage Reclamation | Artifact/enchantment destruction or optional graveyard exile followed by draw |
| Startled Relic Sloth | Trample/lifelink and combat-step optional graveyard exile |
| Syr Alin, the Lion's Claw | First strike and named attack buff for other owned creatures |
| Wreck Remover | Enter/attack optional graveyard exile followed by life gain; cycling |
| Yellowjacket, Heartless Marauder | Flying; other-owned-Villain entry source growth/lifelink |

Validation includes 433 Rust tests (425 unit and eight integration), 76 compiler
tests, 47 rebuilt release Python tests, catalogue consistency, rustfmt and Clippy
with warnings denied. Eighteen native fixtures cover source LKI, private land/
nonland branches, deferred draw/discard triggers, current controller, source
incarnation, target groups/revalidation, fixed N/zero, short hands, APNAP order,
accounting and replay. Fixed N, multiple connivers, opponent-controlled target
and interrupted-source fixtures exercise generic immutable IR; current admitted
printed connive abilities use one conniver and amount one. No partial cards are
admitted to supply those fixtures. Replacement effects and other residual
abilities remain rejected.

One hundred supported Standard games completed naturally: 50,218 decisions,
732 visible connive requests, 732 discard choices, 657 nonland discards, 646
observed immediate connive counter increases and 8,091 cloned/replayed
boundaries. Departed connivers can discard nonlands without receiving counters.
Maximum initialized state was 12,364 bytes for this workload, excluding allocator
metadata, spare capacity, snapshots and static definitions. The smoke policy uses
the actor's observation and varies discard preference for branch coverage. This
is a rules/replay workload, not an XMage speed comparison.

Coverage is 800/5,164 Standard identities, six regression cards and 40 token
factories. Connive replacements, hybrid/variable costs, broader turn restrictions,
conditional/reflexive triggers, control/layers, remaining card semantics and
native RL/search integration are incomplete. Full Standard acceptance remains
outstanding.
