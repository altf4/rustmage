# Creature subtype layer core

`Instruction::SetCreatureTypes` replaces the recipient's creature types for its
current battlefield incarnation. It preserves other subtype classes, including
Food, Equipment, Aura and land types. It removes changeling's earlier creature
types. Resolution records are sparse in the existing optional effect state;
no field is added to each live object. The latest indefinite replacement
supersedes older replacements, and departure erases it. It survives source
movement, ability removal and cleanup.

`StaticAbilityFragment::SourceCounterSubtype` describes one conditional source
addition and its linked keyword fragment. Its source group starts in layer four;
layer-six removal does not revoke the earlier type fragment. Later-layer
keywords still follow ordinary dependency/timestamp ordering. Named Job creature
types use the existing compact eleven-type addition mask; replacement lists can
contain any pinned creature types. This core does not add arbitrary static type
replacement dependencies, card-type changes or copy effects.

Both actual and speculative worlds evaluate the recipient's current counters,
zone, generation and attachment links. Static effects retain entry/attachment
timestamps when their conditions change. Consequently a later replacement
removes an earlier Knight addition, while a newly reattached Equipment supplies
its subtype after the replacement. Frozen ObjectView and DamageFacts retain
this resolved overlay for observations and last-known information. Public
`has_subtype` methods query the entire effective characteristic; overlay
`contains` reports only explicit overlay types.

Observation schema 115 serializes `granted_subtypes` as an object with `added`
and `creature_types` fields. `null` preserves intrinsic creature types; an empty
replacement list removes them all. Intrinsic `Card::has_subtype` remains an
immutable catalogue query.

Skyknight Squire supplies a gated static core fixture. Its entry trigger is not
implemented here and it remains unsupported. Mild-Mannered Librarian likewise
remains gated; its activation, counters and draw are separate consumer work.
Source pins are recorded in `subtype-layer-core-sources.json` and the Squire
provenance. No additional complete cards are claimed by this core change.

Rules references: [Comprehensive Rules, 613.6–613.8](https://media.wizards.com/2024/downloads/MagicCompRules%2004102024.pdf)
and [Foundations release notes](https://media.wizards.com/2024/downloads/FDN_Release_Notes_dPeeQG7XyE/EN_FDN%20Release%20Notes%2008282024.pdf).
