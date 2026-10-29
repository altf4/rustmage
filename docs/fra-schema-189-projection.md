# FRA schema 189 projection audit

Schema 189 adds optional outside-game pools, public sideboard counts and outside-choice observations/actions, plus an optional cumulative free-cast mana-value budget. Extrapolate the Impossible remains gated; general outside-to-hand replacement and observer semantics still need rules evidence.

The guarded library run projected 2,928 existing trace summaries to schema 188. All 2,928 matched their baseline, with no prior-mana exceptions. The projection rejects active or retained outside-game state, outside choices and nested rollback states that cannot be represented faithfully. Independent tests exercise both viewers, actual action history and the composed 189-to-188-to-187 projection. See `src/assurance/regressions.rs` for the guards.

The combined outside-game and budget run passed 2,623 library tests. Its schema-188 trace output was compared directly to the published schema-188 baseline and all 2,928 entries matched. This audit does not establish full card or set acceptance.
