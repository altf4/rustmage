# Projected control and funding bounds

At `f9d9b10e`, the guarded grouped suite passed 2,766 tests and guarded clippy
passed. An independent source review found no definite defect in the narrowed
control guard; final published-batch review and validation remain required.

The previous bound counted current controllers and could reject an affordable
payment when a source-bound control effect ended during a mana-cost departure.
The search now bypasses that bound when a source-linked control effect or a
control Aura is present. It retains the existing bypass for linked returns.
Source-independent control bases and timestamped control assignments do not
alone change a surviving object's controller during a payment, so their mere
presence does not disable the bound. Projected roles and attachment providers
are consulted when present; newly supported entry or control paths must be
reviewed against this assumption.

The both-seat projected regression constructs a source-linked control effect
on a Forest, an Emrakul-style colorless mana grant, and a Lotus Ring attached
to a creature. Sacrificing that creature in the projected funding world ends the
control effect and exposes the Forest's granted mana. The control effect and
grant are injected fixture state; the test does not establish that this exact
setup is reachable through public card actions. A second both-seat test uses 18
Forests and a stable control record with an impossible 1,000-mana goal, checking
the upper bound rejects immediately instead of traversing tap subsets.

This fixes a specific bound unsoundness and avoids an unnecessarily broad
bypass. It does not prove general funding-search termination or cover all
future replacement, projected-entry and dynamically changing provider paths.

Two additional both-seat regressions use public actions after explicitly seeded
starting permanents and mana. The Super Hero Civil War gains control of a Forest
Tentacle; Confiscate and Puppet Crafting make the Saga a controllable creature,
and Lotus Ring sacrifices it during payment, returning the land. One test pays
for Titanbones with the land's ordinary green mana. The other activates
Emrakul's hand ability and pays for Gilded Lotus using the returned land's
specific granted {C}{C} ability. Both repeat raw-action payment after restore,
check stale indexed actions, and accept a fresh indexed cast announcement.
The 2,768-test library suite passed on `5df0dddc`; these scenarios do not certify
Emrakul's general funding termination gate or full indexed payment replay.
