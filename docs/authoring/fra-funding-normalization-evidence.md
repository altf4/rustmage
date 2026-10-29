# Standalone spell-only funding normalization

At candidate `db676001`, a guarded grouped run passed 2,764 tests. Two
independent source reviews found no definite unsoundness under the guard; these
are reviews of this narrow change, not a proof of general funding termination.
Final published-batch reviews and validation remain separate requirements.

The search-key normalization is enabled only without a nested payment
continuation, cast-cost or entry projection, exile links, reserved or enclosing
source, cost/sacrifice group, required face-up object, or convoke. It applies
only to artifact, instant/sorcery, noncreature, planeswalker or subtype
spell-only lots. These lots cannot fund mana activations. Ineligible lots are
omitted from the key. Eligible lots retain their distinct exact qualities and
colors, with each color's key amount bounded by the sum of the fixed final
colored and generic demand. The corresponding key aggregate is reduced by the
same excess. Actual pools, payment allocation, and public state remain exact.

The argument is that, under this guard, these lots have no consumer before the
final payment, and that payment cannot spend more than its total demand.
Color substitution can change requirements into generic but does not increase
that total. This does not justify normalizing mana eligible for activation, or
other lots, or nested/projected payment state. Changing the guard or mana
eligibility rules requires rechecking this argument.

The synthetic prevented-sacrifice regression exercises the formerly unbounded
artifact-only production and an unavailable blue converter; both seats now
finish the actual funding query. Another test retains separate same-color
qualities including a bonus and tests substitution and an exact allocation.
A public-action test taps three real Castle Doom sources for restricted black
mana and casts Aerial Doombot using Vizier of the Menagerie's color substitution,
checking the one-mana payment, retained pool and snapshot replay for both seats.
The latter removes Vizier with a fixture zone move, not a public action; it is
not a real-card witness for the prevented-sacrifice loop.

Unrestricted or activation-eligible positive cycles, changing providers and
qualities, nested payment continuations, and other funding searches remain
outside this equivalence. No general finite-control or termination proof follows
from these results. The synthetic prevention provider remains distinct from an
identified supported real-card provider.
