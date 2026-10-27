# Gandalf, Goblins' Bane // Flameshape

Base22d22c0f9deb9ae5efd1f92529b3e1c3a0372689; isolated complete-card candidate, pending two independent nonauthor reviews and root integrated release acceptance. No PR/push/merge. Root reserves5417 exclusively for the new Flameshape Adventure face and regenerates central links. Existing physical1676 is pinned to its actual2R legendary2/3 Avatar Wizard front; joined identity remains an alias and typed Adventure identity. Flameshape is1R sorcery Adventure, parent1676.

Full Oracle identityf48f2a9b-4ba5-4984-8180-3950889bba65 and both faces read. Complete pinned XMage GandalfGoblinsBane.java (including both in-file effects), AdventureCard, SpellCastControllerTriggeredAbility, BoostSourceEffect, DamagePlayersEffect, MayLookAtTargetCardEffect, PlayFromNotOwnHandZoneTargetEffect, AsThoughEffectImpl, FixedTarget, FixedTargets and FilterControlledCreaturePermanent reviewed at492369c7844b8d76479f0393ff9eb6b0b0cf30d7. Both paired records pin Java/dependencies, full Oracle facts, exact face facts and physical metadata. Oracle says a Wizard; the typed guard reads current controlled permanent subtype rather than requiring printed creature qualities from XMage's creature filter. No English parser.

## Obligations and shared support

| Printed obligation / boundary | Independent outcome |
| --- | --- |
| Paid creature |2R spent for2/3 Avatar Wizard; noncreature trigger not self-triggered |
| Every own noncreature cast |+1/+1 EOT and1 actual damage to opponent; original countered spell still triggers; own creature/opponent noncreature do not |
| Independent effects / source | Source departure and reentry do not grow new incarnation, but old packet still deals damage; lifelink proves damage, not life loss |
| Trigger multiplicity/copies | Staff doubles whole packet and its separately granted prowess; a copied spell is not cast |
| Current source abilities/control | Suppressed source does not create new packets; current controller's casts do |
| Paid Adventure |1R spent; source follows ordinary Adventure exile/creature-cast rights; countered Adventure does not exile library cards but does trigger an existing Gandalf |
| Exactly top two, private |0/1/2 and3-card library boundaries; only selected actual exiles are hidden; caster sees names, opponent sees neither objects nor permission/linked references |
| Persistent look / conditional play | Look remains without Wizard, after source departure, and through real turn cleanups; paid spell/land play requires current Wizard control/type |
| Current conditional facts | Source's own later Adventure body qualifies; stolen Wizard ceases to qualify; changed Bear subtype qualifies; Witness makes Wizard cease/reacquire |
| Normal play restrictions | Costs, timing, land allowance, incarnation and ordinary ownership/control rules remain; cast and land play publicly reveal selected card |
| Locked approval | Actual Ring sacrifice of last Wizard during mana payment does not revoke approved cast/locked cost |
| Owned / transaction | Private exact permission raw/index/restore/parent disposal, invalid actions, actual foreign-Aura failed cast restores hidden row and permission with retained/reversed mana |

Front reuses existing SpellCast/Noncreature, Grow Source and DamagePlayers typed rules. Flameshape's private ExileTopFaceDownPlay instruction freezes top two, performs simultaneous raw movement, and adds rights only for successful final exile. It reuses sparse ChoiceState.hidden exact-object/viewer masks. Standalone hidden rows now keep ChoiceState alive through abandon_linked_choices; ordinary pruning already uses exiled incarnation, not source lifetime. Source is merely an association key and never a future play/view prerequisite.

Existing ExilePlayPermission gains `requires_subtype: Option<&'static str>` (None omitted), tested as current controlled battlefield subtype in paid spell, land, modifiers and direct alternate-Adventure origin paths. Existing row constructors use None. Casting permission is checked before announcement; existing locked total/cast proposal survives last-Wizard mana sacrifice under CR601.2e–h. Observation's permission rows now respect existing hidden-object visibility, avoiding otherwise public exact references. Look access remains independent of Wizard condition. No new State field/action/ID other than reserved5417; existing hidden-vector and permission-row initialized-byte accounting includes new payload, and snapshots/owned branches retain it.

New public conditional-row vocabulary requires166 or later after in-flight165 checkpoint; central schema is deliberately unchanged. Existing None rows serialize unchanged; root owns inherited-row audits/adoption. Peer ownership coordinated directly: Thorin/Dáin reflexive attachment and LKI work, Down entry/self-granted templates remain disjoint.

Rules authority: official20260925 CR cached `<cache>/rules/MagicCompRules-20260925.txt`, SHA2568d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca. CR400.7 exact objects; CR406.3 face-down exile/look permissions; CR601.2 legality then locked costs/mana payment; CR707.10 copied spells are not cast. Existing hideaway uses this same private representation. No admitted exile-object typed targeting consumer is added.

## Validation

Final coherent focused run passed99 executions/99 unique scenarios,0 failures/ignores and no warnings. Stable dedicated `<cache>/hob-gandalf-flameshape/target` and author test opt0, default integrated release profile unchanged. Persistent driver, whole guard outer stdout/stderr, per-group logs/results/resource from launch. Inputs fixed through natural terminal, no cancellations/active-output deletion. Source fixture names/costs/admission preflighted; Staff's additional prowess is included in expected stats. Local `mod hob_gandalf_flameshape;` and generated links excluded from commit. Committed rollback registration at sos_mana_rollback.rs tail uses `#[path="hob_gandalf_flameshape_rollback.rs"] mod gandalf_flameshape_tests;`.

Initial run `<cache>/hob-gandalf-flameshape/run-80e4487c918e4540ae3b96e13dacc6f6` naturally terminated101 (session57305):14 main scenarios passed; the one rollback fixture stopped before setup payment because Silverquill grants the Adventure an optional casualty choice. Explicitly declined that setup casualty only after terminal. No runtime source fix. Guard106.733315762s including queue, peak2682138624bytes/all OOM0. Full family rerun from coherent fixed inputs; no cancellation.

Final receipt `<cache>/hob-gandalf-flameshape/run-7520ec8d3536440e95a0311a8decfa17/`: original session33224 naturally exited0; all inputs fixed from launch to terminal. Groups: own15 (14 main+1 real rollback), hideaway/library11, Decree13, Inside16, Bilbo16, mana rollback3, cast rollback6, extra triggers14 and storm5. All nine commands returned0/no timeout/orphan. Compile30.41s, guard36.866701884s including queue, peak2636259328bytes within6442450944byte cap, one job/thread, all OOM counters0. Native reported1.02s combined; this bounded family is not a whole-game/performance acceptance claim.

Grouped command elapsed32.167089196s. Final source pins5418 definitions/1620 Java blobs, Rustfmt and whitespace checks pass. The earlier fixture failure remains documented above; no runtime rules change was needed. Root still owes two independent nonauthor reviews, coordinated schema166+ adoption/inherited audits and integrated full release acceptance.

Receipt SHA256:

- `driver.py`: `1108b9a14d3d4e5186a98a6555a8be2accc780d5cf8c9b48bd38ac39727daa00`
- `results.json`: `1e4f64450446fad97c03b82d7c446eeff4cda45183ae95127ea4b0bd6ccbf07f`
- `resource.json`: `3b8fce2698c1363f8641c1551be8e5048b2eb59a205486a44de8247f10b85e86`
- `outer.log`: `244688eb80d3248d0e60547ec8b152b2800333af5b1a37de3b23bdfd8afa5377`

Authored Rust SHA256:

- `src/card_definitions/hob/gandalf_goblins_bane_flameshape.rs`: `6f54a8e8e4fb2c57c4439d09d203a961620d4ec66c83e14da8a9823857aa7f68`
- `src/card_definitions/synthetic/flameshape.rs`: `49bdc5fb210f83998ae377779fcb7acc332ff9b667acd7d8f009dfd142aaba8d`
- `src/assurance/hob_gandalf_flameshape.rs`: `e68ec9a3d4629a78d78820055935377609ede40e75e70c37a5b36ab34dcae4f1`
- `src/assurance/hob_gandalf_flameshape_rollback.rs`: `8ac96315b49396788d336a9fd4e9ed4217a4ee13061c4003395da7396e0df8fb`
- `src/engine_exile.rs`: `bd889eae26fd33ba6679f5fcdf5d8f999bb816968255180102df1f7efa3b4887`

Implementation interval from isolated checkout creation to final receipt recording:1139.7s for one complete physical card/two spell definitions plus conditional private exile rights, including source review, authoring, queue/build and two guarded runs. This is candidate author throughput, not merged completion or simulation throughput.
