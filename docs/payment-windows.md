# Casting and activation payment windows

An admitted spell or ability can now be announced before its mana is floated.
AnnounceCast, AnnounceActivate and AnnounceMana retain a captured source, fixed
total cost, selected counter payment and intended production in owned state.
Targets are selected first. The engine then opens pay_costs, where the actor can
activate eligible mana abilities and commit an exact PayCosts payment. Existing
funded atomic actions remain available as fast paths.

This permits choosing a Treasure as Disenchant's target and then sacrificing
that Treasure for white mana to pay Disenchant. Its target retains the original
incarnation. The token remains in its destination until casting completes,
then ceases at the next SBA; Disenchant later fails to resolve because its only
target is illegal. Prefunding alone could not represent this sequence.

No priority, responses, intermediate state-based checks or trigger placement
occur inside the declaration. Cost-generated triggers queue until the
declaration commits, then are placed above the completed spell/ability. Activated
mana abilities still resolve immediately; announcing one with an unpaid mana
cost can use other mana abilities to fund it. The same pending mana ability
cannot recursively activate itself (CR605.3c).

Source obligations reserve only costs that actually need the original object:
tap, sacrifice and counter removal. An ability with only mana/life costs remains
payable if its source leaves during funding. Source context and costs stay
captured; a new incarnation cannot pay an old object's source costs.

## Feasibility and legality

The action mask offers an announcement only if full payment is reachable.
During payment it offers only mana choices that preserve a complete payment.
The solver includes exact colored/colorless/generic costs, mana conversions,
tap/sickness/haste, self-sacrifice, life/counters, use limits and reserved source
obligations. It does not count queued triggers as available mana.

Reachability uses compact temporary resource copies and memoization, not full
game clones. Equivalent sources are canonicalized while the enclosing source
remains distinguished. A conservative upper bound can reject impossible plans;
production ordering only changes traversal order. There are no depth or node
cutoffs that silently remove legal plans. Already funded goals use a cheap
direct check. New unbounded resource-free positive mana production stays gated
until an infinite-resource solver exists; finite conversions are supported.

A regression compares compact reachability against exhaustive native mana
sequences on a small board across 54 cost combinations. Other fixtures exercise
source reservations, nonactive-player priority, illegal stranding choices,
token cessation timing, queued triggers, source departure and replay from each
casting stage.

## Interface and remaining scope

Schema **11** adds the announcement/payment actions and actor-only
payment_request (required mana, generic amount, selected counters and intended
production). Opponents see no partial payments or target choices. Snapshots own
every declaration field; feasibility queries do not change state, revision or RNG.

This strengthens rules enforcement for the existing 212 executable Standard
definitions; it adds no new card identities. Broader costs, alternative costs,
modes, payment requests from unsupported effects, triggered/restricted/variable
mana, infinite resource production and layers remain outstanding.

Rules: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
especially 118, 601.2b–i, 602.2b, 605.3 and 704.
