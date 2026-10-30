# Schema 186 historical projection

Schema 186 adds Garruk's next-turn stat duration and Chandra's full-width loyalty X and restricted mana. The schema 185 projection remains a separate, unchanged step for Pyre. The new preceding guard rejects those new fields and identities in public observations and typed hidden state, including copy recipes, nested packets, mana qualities, active stat durations and rollback checkpoints. Opaque continuations and mana receipts are conservatively unrepresentable. Unchanged schema 185 observations are preserved byte-for-byte.

The guarded combined candidate run recorded 2,924 supported-card trace summaries. All 2,924 matched the schema-185 baseline after projection; the current-schema baseline was recorded from the same run. Five new scenario failures remain under investigation, so card admission remains gated.

The 273-card candidate retained all 2,924 existing current-schema trace summaries and all 2,924 schema-185 projections. Two new summaries correspond to Jace, Reality Sculptor and Pyre Rhymer // Molten Tide. The candidate run passed 2,543 tests; one new animation fixture needed correction before the final rerun.

After correcting the animation fixture and recording the two reviewed admissions, the guarded library suite passed 2,545 tests; the full-set game check reported a non-acceptance skip while the pool remained incomplete.

The 275-card candidate passed 2,544 behavioral tests; its only failure was the expected addition of Chandra and Garruk traces. All 2,926 prior current-schema summaries matched. Full-set gameplay remains unverified.
