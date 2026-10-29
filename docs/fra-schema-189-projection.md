# FRA schema 189 projection audit

Schema 189 adds optional outside-game pools, public sideboard counts and outside-choice observations/actions. Extrapolate the Impossible remains gated; general outside-to-hand replacement and observer semantics still need rules evidence.

The guarded library run projected 2,928 existing trace summaries to schema 188. All 2,928 matched their baseline, with no prior-mana exceptions. The projection rejects active or retained outside-game state, outside choices and nested rollback states that cannot be represented faithfully. Independent tests exercise both viewers, actual action history and the composed 189-to-188-to-187 projection. See `src/assurance/regressions.rs` for the guards.

One player-control scenario failed and is under investigation. This audit does not establish full card or set acceptance.
