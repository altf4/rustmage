# Tenured Tethermage: source as a tap-cost candidate

The draft card remains incomplete. Its activated ability requires tapping two
untapped artifacts controlled by its controller, without a separate tap-symbol
cost and without excluding the source. If an effect makes the Tethermage an
artifact, its controller may choose it as one of those artifacts.

The current additional-cost implementation unconditionally excludes the
activation source for `TapPermanents` in candidate funding, selection, and
commitment. Before admitting this card, account for whether the activation
also has a tap-symbol cost and whether the filter excludes the source; make
funding projections and payment agree. Verify source animation, an already
tapped source, other-filter exclusions, an ability with a separate tap cost,
competing mana resources, duplicate choices, transactional failure, and
branch/restore. The draft and its provenance are not proof of completion.
