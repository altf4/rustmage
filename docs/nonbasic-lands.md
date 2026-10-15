# Nonbasic lands and pre-entry conditions

Lands are played directly from hand at sorcery timing, using the normal one-land
allowance. Playing one does not cast a spell or use the stack. Entry triggers
still use the stack and normal targeting/ordering. Nonbasic lands use the existing
activation interpreter and exact mana funding solver: unrestricted fixed/choice
production, colorless mana, generic conversions, tap and self-sacrifice costs,
sorcery-only abilities, scry/surveil and other complete supported instructions.

Immutable EntryTapped definitions describe self replacements: always tapped,
unless controlling a basic land, unless controlling at most two or at least two
other lands, and unless either player has 13 or less life. Wastes has the Basic
supertype, no basic land type and no four-copy limit. Basic land types on admitted
surveil lands confer their intrinsic mana abilities; those are compiled from
validated reminder text, before ordinary reminder removal.

Every real battlefield entry applies its condition, including returns from other
zones. A simultaneous move captures entry decisions against the pre-event
battlefield before moving any objects. Paired exile returns do the same after
the original source-departure event. A simultaneously arriving basic land or
third land cannot change another arrival's decision. Conditions do not retap or
untap a land later when the battlefield or life totals change. There is no extra
mutable cache or snapshot field; existing object tapped state and immutable
catalogue rules suffice. Schema 13 is unchanged.

Whole-text admission adds 135 Standard identities (499/5,164 executable), covering
fastlands, slowlands, low-life duals, gainlands, Guildgates, tri-lands, temples,
surveil lands, damage deserts and selected utility lands. Source references include
XMage BloomingMarsh, DeathcapGlade, AgnaQela, CommercialDistrict, CaptivatingCave,
AbandonedCampground and Wastes. Unsupported entry choices, shocklands, restricted
mana, fetch/search, land animation, legendary lands and residual rules text stay
gated. These remain implementation work toward the full Standard target.

Native tests cover play timing/allowance, tapped funding exclusion, exact colors,
threshold boundaries and ownership, Wastes, entry reevaluation, simultaneous
moves and paired returns, life/damage triggers, private surveil choices,
conversions, sacrifice costs and replay. Python exercises tapped play, private
surveil, untap, exact mana projections and clone/restore. The standard_lands
example admits two complete 60-card supported Standard decks with nonbasic mana
bases and plays them through the public library API.

Rules: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
especially 305, 603, 605 and 614.12. This implements the listed self replacements;
general replacement ordering, dependency and choice machinery remain outstanding.
