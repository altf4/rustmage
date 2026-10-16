# Contextual cast qualifications

Issue: https://github.com/altf4/rustmage/issues/195

On the standalone branch based on qualified-cast PR194, coverage is **1,401/5,164**
pinned Standard cards, with **3,763 unsupported**. Six non-Standard regression
identities, 57 token factories and one emblem remain separate. The full Standard
migration remains incomplete, pinned to October 15, 2026. Integration with other
independent branches changes aggregate counts and the catalogue fingerprint.

This stage implements seventeen complete bodies: Colleen Wing, Street Samurai;
Danitha, Spear of Agony; Danitha, Sword of Hope; Firespitter Whelp; Forum Necroscribe;
Gnarlback Rhino; Graduation Day; Informed Inkwright; Inkling Mascot; Inkshape
Demonstrator; Lecturing Scornmage; Madame Hydra; Melancholic Poet; Mockingbird,
Ace Agent; Rehearsed Debater; Rodeo Pyromancers; and Stirring Hopesinger.

## Completed casts and targets

Cast observers run only after all modes, targets, mana and additional costs have
completed. The contextual filter reads the completed stack item's existing bound
target slice, without cloning its targets or adding mutable state. Existing indexed
cast observers, ownership, public guards, once-per-turn limits and source snapshots
remain shared with ordinary cast events. Activations and proposals are not casts.

The new immutable `CastFilter::Targeted` composes a printed spell filter with a
current-target requirement: a creature you control, any creature, an opponent or
opponent creature, or the exact source. Existing `AnyOf` supplies Equipment OR
owned-creature-targeting and noncreature OR Dragon qualifications. Villain spells
include changeling spells. Rodeo uses the existing owned cast ordinal with value 1;
an unrelated first spell consumes that ordinal, and counts reset each turn.

A spell with several qualifying targets triggers each ability once. Danitha, Sword
of Hope's per-turn limit applies across both arms of its union. Repartee requires
an instant or sorcery targeting a creature of either player. Auras can satisfy the
broader owned-creature requirement but do not satisfy Repartee. The Repartee label
is recognized only with its entire exact header. Storyteller Pixie remains gated
because Adventure casting is not implemented. Unknown qualifiers, combined events,
pronouns, tails and residual abilities keep the whole card unsupported.

### Rules correctness and the pinned Java divergence

[CR 115.9b in the pinned official rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf)
says a condition looking for a spell that targets something checks its targets'
current information in the expected zone. An object that has left that zone is
ignored; its last known information is not used. Accordingly, a creature selected
as a spell target and then sacrificed as an additional cost does not qualify at
cast completion. Returning the same card ID does not restore the old target:
the bound incarnation must still match. Target legality is a separate question;
gaining hexproof after target selection does not erase an otherwise matching
current creature target.

The pinned Java `TargetsPermanentPredicate` uses
`getPermanentOrLKIBattlefield`. Rust deliberately does not copy that behavior for
departed targets. This is an explicit rules-correctness choice, consistent with the
request to prioritize legal gameplay over XMage compatibility. Opponent-player and
source-self requirements remain separate from creature filters.

Once a cast has triggered an ability, removing or countering the spell does not
cancel that ability. This also follows the official
[Secrets of Strixhaven release notes](https://magic.wizards.com/en/news/feature/secrets-of-strixhaven-release-notes)
and [Marvel Super Heroes release notes](https://media.wizards.com/2026/downloads/MSH_Release_Notes_N6hMZoes90/zfGb4BJ8Bf_EN_MTGMSH_ReleaseNotes_20260513.pdf).
A captured ability retains its controller and source incarnation; self counters
and temporary bonuses cannot be applied to a returned incarnation. Draw, life,
token creation and private library choices remain owned resolution continuations.

## Complete bodies and library behavior

All intrinsic keywords, legendary status, costs, types and extra abilities are
retained. In particular Forum keeps discard Ward and exact owned-graveyard
reanimation, Inkshape keeps mana Ward, Colleen's counter precedes scry, Inkling's
flying precedes surveil, and Stirring affects every current owned creature.
The existing Inkling and Villain factories retain exact statistics, colors and
keywords; no token factory is added or renumbered.

`contextual-cast-admissions.json` records all seventeen complete Oracle bodies and
**140 SHA-256 entries across 58 unique Java files**, pinned to revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Constructor and shared target/cast/filter
logic was inspected; recorded blobs match both that revision and the checkout.
Exactly these seventeen immutable definitions change; the other 5,224 definitions
and all card/token/emblem identities remain unchanged on the standalone base.

There is no new game-state or observation field. Schema stays **52**, C ABI **1**
and control protocol **2**. Initial logical state remains **19,848 bytes** on this
base, excluding capacity, allocator overhead, immutable definitions, observations
and separately retained snapshots. Restoring invalidates stale action indices;
capacity failures restore the entire action, including cast counts and payments.

## Acceptance

Twelve native tests cover both-seat complete casts and effects for all bodies,
real target selection and additional sacrifice costs, selected-target versus completed
cast timing, exact incarnations, later target illegality, multiple qualifying targets,
ability exclusion, Equipment union limits, ordinal reset, opposing casts, Aura versus
instant/sorcery, real Counterspell, source departure and atomic capacity failure.
Three compiler tests cover exact headers, wrappers, unknown residual rejection and
all whole-body admissions. Earlier assertions marking Rodeo, Stirring and targeted
instant/sorcery events unsupported were updated without relaxing unknown qualifications.
The Python test executes seventeen complete-cast/effect scenarios through the C ABI,
including real Forum reanimation, private scry/surveil, snapshots, clone replay and
stale indexed-action rejection.

`examples/standard_contextual_casts.rs` uses legal 60-card Standard decks and an
independent public ledger for cast/target qualifications, source-incarnation trigger
debts, per-turn limits, counters, temporary statistics/keywords and each complete
effect. It witnesses private frames and both players' public statistics. Prefix
effects already applied before a suspended library choice are checked immediately.
Mandatory Forum or Graduation triggers without legal targets are accounted for
separately. Replay runs use one catalogue fingerprint throughout.

Two 100-game release runs reproduced **59,390 decisions**, casts
`[29,32,39,30,13,21,28,33,24,41,41,32,33,28,31,27,35]`, triggers
`[3,6,9,27,3,3,26,7,8,25,5,13,5,1,15,36,28]`, verified effects
`[3,6,9,26,3,3,20,7,8,25,5,13,5,1,15,36,28]`, **1,908 cast/observer checks**,
11 private choices, 13 Ward packets, three triggers without legal targets,
**121,463 public stat/keyword/counter checks**, **40,599 replay boundaries** and
**21,204 maximum initialized logical bytes**. Arrays follow the card order above.
Differences between trigger/effect counts include impossible target announcements
and terminal games. Fingerprint:
`3ff311be8ddd55f122a63c0ac1b1cdc89fdcfb871137db6134d0a9e9e3932974`.

The 1,000-game smoke benchmark measured 571,049 decisions, about 318,314 decisions/s,
557 games/s, 128 ns clone and 132 ns restore on this host. Measurements include policy
and observation construction, exclude model inference, tensor encoding and JSON,
vary with concurrent work, and are not an XMage parity comparison.

Full standalone validation passed: **1,223 Rust tests**, **233 compiler/data tests**,
**152 fresh release Python tests**, catalogue consistency, formatting, all-target
Clippy with warnings denied and diff checks. Independent PR reviews and integrated
validation are recorded separately after integration.
