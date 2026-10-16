# Mandatory additional spell costs

Issue: https://github.com/altf4/rustmage/issues/84

The native announcement flow supports one mandatory discard or filtered sacrifice
alongside the spell's mana cost. Complete new Standard definitions are Corrupted
Conviction, Final Vengeance and Thrill of Possibility. Coverage is 910/5,164,
with six regression definitions and 41 token factories. Unsupported alternatives,
optional costs, multiple-object costs and residual mechanics stay gated.

Callers use `AnnounceCast`, choose modes/hybrid equivalents/targets when needed,
then `ChooseAdditionalCost { object }`, mana actions and `PayCosts`. Direct `Cast`
and `BeginCast` are unavailable for these cards. The selected incarnation must
belong to the caster and satisfy its zone/type filter. The spell cannot discard
itself. A target may be sacrificed to pay, making that target illegal on resolution.
Payment precedes completed-cast events, following CR601.2b–i in the
[pinned Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).

An exact funding search reserves the selected object throughout mana payment.
It may tap for mana and subsequently pay a sacrifice cost, but cannot also
sacrifice itself for mana. Source obligations and additional reservations have
separate canonical-key flags. Reservations retain otherwise irrelevant hand or
battlefield objects, validate generation/owner/filter even on the prefunded path,
and recompute effective keywords as virtual sources depart. Ordinary spells keep
the existing shared mana-cost feasibility cache; additional costs bypass it.

Costs may synchronously return linked exiled permanents. Any required Aura
attachment or pre-entry life choice completes before casting finishes. The paid
announcement remains owned state during that suspension; resumption pays neither
mana nor the additional cost again. Triggers caused by payment wait until the
spell is cast and are put on the stack above it using existing APNAP ordering.

Observation schema 34 adds `additional_cost` for the caster only, containing the
kind, exact candidate references, selected reference and paid status. Candidate
lists appear during `additional_cost`; hand selections stay private until the
actual discard. The synchronous Rust API, C ABI and Python binding share the
same legal-action path. Snapshots and clones own every selection/continuation.

Tests cover missing/wrong/stale objects, direct-cast/payment bypasses, target
fizzle, prefunded reservations, tap-then-sacrifice, double-spent Treasure,
discard privacy/events, returned Aura/land entry and stale-revision replay.
Some funding/link fixtures are explicitly synthetic; they do not admit other
printed cards. `standard_additional_costs` runs supported 60-card Standard decks
with all three new cards and checks payment plus clone/restore boundaries.
This milestone is not full Standard acceptance or a Java speed comparison.
