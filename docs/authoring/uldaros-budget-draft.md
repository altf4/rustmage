# Uldaros cumulative cast budget draft

The optional `ResolutionFreeCasts.remaining_mana_value` field is an inclusive
budget for an immediate instruction to cast without paying mana costs. The
budget is checked against the selected face and consumed when a spell commits,
not when its announcement starts. X is zero on this free route. Paid and other
alternative-cost routes are deliberately excluded until their chosen X can be
validated transactionally before commitment. A zero budget still permits a
zero-mana-value spell.

This field is observable whenever the containing free-cast offer is observable;
`None` is omitted, preserving existing JSON for other offers. Before integration,
assign the next schema version and reject prior-schema projection for every
observation, historical action, active owned continuation, and rollback checkpoint
containing a nonempty budget. An empty/absent field retains the previous shape.
The draft tests are unlinked and unbuilt, and exercise a synthetic permission
within an actual Chandra continuation. The tests include a synthetic checkpoint and an actual interrupted additional-cost
payment caused by an opposing entry choice. The latter does not exercise mana
replay; a case with irreversible or selectively reversible mana events remains
necessary. No Uldaros behavior is admitted by this change. Its target selection,
exile replacements and shared copied-card lifecycle remain separate prerequisites.
