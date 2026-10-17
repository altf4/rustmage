# Predefined Role Auras — issue 282 (draft)

This branch adds the six Standard Role factories and twelve complete consumers,
stacked on the ownership/controller work in PR284. It is not merge-ready.
The remaining 22 Role consumers retain their whole-body gates.

The runtime stores Role timestamps in sparse owned game state. Token replacement
finishes before Role tokens enter attached; other replacement-generated tokens
enter normally. Cursed sets base statistics after characteristic-defining effects
and before counters, temporary additions, attachment bonuses and static modifiers.
Young Hero grants its attack trigger without a power/toughness bonus.

Roles controlled by different players coexist. A controller keeps their newest
Role; simultaneous newest Roles require a public, exact-reference choice.
Role and legend choices use the same frozen APNAP state-based event, preserving
simultaneous lethal deaths, counter cancellation and immutable trigger provenance.
The choice and timestamp records are included in snapshots and schema 84 observations.

Royal Ward, Sorcerer scry and Young Hero's intervening-if trigger belong to the
enchanted creature's controller. Wicked's graveyard trigger belongs to the Aura's
controller. Existing immutable granted-ability packets survive grantor departure.

Validation so far: nine native groups pass for all six factories, both-controller
coexistence, timestamp replacement and simultaneous choice, real Doubling Season,
paid Adventure spell/normal-face casts, Ward payment/decline and detached-source
attack triggers with privacy and intervening-if checks. The ninth group passes the eight ordinary consumers through paid casts and activations.
The complete native suite passes 1,721 unit tests plus 12 integration tests. Compiler controls
cover renamed whole programs, authenticated raw reminders and unknown residuals.
The semantic catalogue proof preserves 5,299 prior definitions, all 817 prior helper
arms and all 5,311 prior color facts; twelve complete definitions change, four
Adventure helpers and six token recipes are added. Prior token IDs can shift.

Pending: broad compiler/API gates, fresh public Standard
acceptance, remaining layering/illegal-attachment boundaries, final provenance
review, and two independent nonauthor exact-head reviews. This does not claim that
all Standard rules, control-changing effects or all Role consumers are complete.

Sources: [official WOE release notes](https://magic.wizards.com/en/news/feature/wilds-of-eldraine-release-notes),
[pinned comprehensive rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf)
(CR303.7, 613 and 704), and `role-admissions.json` with the 34-card inventory,
six predefined factories and 43 full pinned Java source hashes. This inventory
never supplies compiler admission permissions.
