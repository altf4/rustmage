# FRA schema 190 projection audit

Schema 190 changes stack source representation to an optional owned source, adds player counter and proliferate observations, and records Uldaros's explicit target-type slots. Tam, the Possibility and Uldaros Theorix remain gated pending complete rules and source evidence.

The combined guarded library run at tested draft `aa3a9e6e` passed 2,683 tests. The previous-schema output was compared directly with the published schema-189 baseline at `0e8ab675`; all 2,928 entries matched. The projection restores the prior serialized source field order for representable packets and rejects source-less rules, player counters, target-type slots, and other retained state that cannot be represented faithfully. Tests include actual packet bytes, nested copies, both viewers and owned rollback.

This is compatibility and interaction evidence, not full set acceptance. Final validation and nonauthor reviews remain required after subsequent changes.
