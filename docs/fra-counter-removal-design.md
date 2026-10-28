# Reality Fracture: target counter removal

Mabel, Bitter Recluse requires a choice of zero through three counters from one
other creature or planeswalker. Different counter kinds may be selected. The
choice is made when the triggered ability resolves, after its target has been
revalidated. Selection is an incremental interface to one rules choice: no
counter changes and no state-based actions occur between selections. Finishing
removes the selected counters together. The partial selection is visible to the
decision maker and players sharing their information; the committed counter
changes are public.

The draft owns the target incarnation and selection in the resolution state.
It permits only counter kinds and quantities currently on the target and
rejects choices beyond the maximum. Cloning, restoring and failed action
validation must preserve the suspended selection. Schema 171 is tentative and
must be reconciled with the independent scry and outside-game drafts before
integration; a single final bump should cover all changes published together.

Hexproof counters are appended to the existing named-counter and keyword-counter
sets so older counter ordering is preserved. This is necessary for Mabel's
interaction with Leatherhead, Swamp Stalker. Existing layer-six timestamp rules
apply when such counters are added or removed. Counter removal currently uses
the engine's existing counter-removal primitive; a source audit found no
counter-removal observer in the pinned Standard Oracle, but full-pool
interactions and replacement rules remain subject to the final audit.

The card remains gated. The authored tests cover mixed choices, the zero choice,
planeswalker loyalty, illegal and wrong-actor actions, source and target
departure, hexproof, hidden staged state, and raw/index branch and restore. No
Rust test in this draft has run yet. In particular, compilation, type-changing
targets, replacement interactions, FFI behavior and performance still require
execution and review before admission.
