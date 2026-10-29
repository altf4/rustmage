# Pyre Rhymer historical projection

Schema 185 adds public Mountain-mana bonus fields on players, retained spell packets and mana rollback views. Schema 184 traces remain byte-identical when the new behavior is absent. The guard rejects Pyre and Molten Tide identities, those public fields, hidden player bonuses and retained packet bonuses. It traverses copy recipes, captured packets, grants, delayed effects and rollback checkpoints; opaque active resolutions, replacement continuations and mana receipts are conservatively unrepresentable, including receipts with no Mountain bonus, because their historical replay cannot be established from these checks. Existing schema 184 paid-cast observations remain representable when their owned continuation is otherwise inspectable.

The guarded candidate run recorded 2,924 supported-card traces; all 2,924 projected trace summaries matched the schema-184 baseline. The schema-185 trace baseline was recorded from that same run. The candidate run also exposed three Pyre scenario failures, so card admission remains gated pending their fixes and a passing rerun.

After the three Pyre fixtures were corrected, the guarded library suite passed 2,509 tests with the full-set game test still ignored. This is candidate evidence, not full-set acceptance.
