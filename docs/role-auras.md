# Predefined Role Auras — issue 282

This branch adds the six Standard Role factories and twelve complete consumers,
composed with the ownership/controller, mill/recovery Adventure and graveyard recovery deliveries.
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
The choice and timestamp records are included in snapshots and schema 85 observations.

Royal Ward, Sorcerer scry and Young Hero's intervening-if trigger belong to the
enchanted creature's controller. Wicked's graveyard trigger belongs to the Aura's
controller. Existing immutable granted-ability packets survive grantor departure.

Validation passes: 1,748 native tests and all-target Clippy; all 420 unique
compiler/provenance methods; a 254-method fresh public checkpoint plus the newly
added actual Tombstone-payment method run separately on identical production;
and 100 natural Standard games with all 24 Role body/seat vectors, 81,882 owned
replays and 163,764 wrong-actor/stale denials. The canonical composition preserves
5,306 prior definitions, all 843 prior helper arms and all 5,318 prior color facts.
Twelve complete definitions change; four Adventure helpers and six factories are
added. The [review record](review-pr-289.md) identifies exact runtime, policy and
test-source scopes, historical failed attempts, independent adversarial probes
and required final exact-head approvals.

This is 1,862/5,164 executable Standard identities, with 3,302 still gated. It does
not complete all Standard rules, all Role consumers, changing-control effects or
the full RL/search interface and workload-performance requirements.

Sources: [official WOE release notes](https://magic.wizards.com/en/news/feature/wilds-of-eldraine-release-notes),
[pinned comprehensive rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf)
(CR303.7, 613 and 704), and `role-admissions.json` with the 34-card inventory,
six predefined factories and 43 full pinned Java source hashes. This inventory
never supplies compiler admission permissions.
