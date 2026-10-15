# Typed counters and untapping

Objects store three counter counts: +1/+1, -1/-1 and stun. Public
`ObjectView.counters` was introduced in observation schema **7** (retained in schema 8). Counts are owned
snapshot state, and resetting a zone incarnation clears them. Immutable
instructions place counters on bound targets or the original source incarnation.
An ability resolving after its source leaves does not modify a returned object.

Power/toughness includes counter modifiers separately from temporary growth.
Opposing +1/+1 and -1/-1 counters cancel during state-based actions, simultaneously
with creature deaths; dying objects retain their pre-batch last-known
characteristics. Cleanup checks for deaths exposed by expiring growth and
provides an extra priority cycle before advancing the turn.

A stun counter replaces each attempted untap of a tapped permanent with removing
one stun counter. Already-untapped permanents keep their counters. The untap step
and effect instruction use the same implementation. Multiple stun counters
therefore require multiple attempts.

The compiler accepts only complete, exact supported clauses. Eight added
Standard admissions are Bile-Vial Boggart, Blight Rot, Ironpaw Aspirant,
Jeong Jeong's Deserters, Machinesmith Automaton, Sazh's Chocobo, Sterling Supplier,
and Utrom Scientists. This milestone brought coverage to **114/5,164**. Entry counters,
counter transfer/removal/doubling, new counter kinds, counter-placement triggers
and replacements, and unsupported additional text still block admission.

Rules reference: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
especially 122.1a, 122.1d, 122.2, 704.5 and 514.3.
