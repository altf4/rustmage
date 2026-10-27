# Bilbo, Thief in the Night

Base: `243d8b943ac39b74b7fe9ec1c4aa62ea394b40e4`. Direct typed definition, paired source record and disjoint `hob_bilbo` / `hob_bilbo_rollback` assurance. Candidate only; root owns integration, generated links, release acceptance and two independent nonauthor reviews. No PR/push/merge.

Oracle identity `72a9a85d-7cfc-4b3c-82db-9d35be6c0982`, catalogue411: {1}{U} legendary2/2 Halfling Rogue. All nonhand spells receive one generic reduction; attack permits one optional paid artifact/instant/sorcery cast from the controller's graveyard during resolution. Only an instant/sorcery actually cast through that permission receives the exact stack-incarnation graveyard exile replacement.

Complete pinned XMage `BilboThiefInTheNight.java` and its in-file effect reviewed at492369c7844b8d76479f0393ff9eb6b0b0cf30d7. Full focused dependencies read: AttacksTriggeredAbility, SimpleStaticAbility, SpellsCostReductionControllerEffect, CostModificationEffectImpl, ThatSpellGraveyardExileReplacementEffect, SpellCastFromAnywhereOtherThanHand, TargetCardInYourGraveyard and FixedTarget. CardUtil's entire `castSingle` overload chain was read (utility file hash pinned; unrelated utility methods are not claimed as this card's rules proof). Java chooses using the physical card and installs replacement by its pre-cast kinds; Rust checks the actual chosen spell face under CR601.3e. This allows Clap! Snap! without allowing its creature front. Java's replacement duration is end-of-turn; the existing exact stack-incarnation replacement is the Oracle obligation, independent of a turn deadline.

Rules authority: official20260925 CR cached at `<cache>/rules/MagicCompRules-20260925.txt`, SHA2568d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca. CR608.2g supplies timing and no intervening priority; CR118.9 permits ordinary alternative costs and applies reductions/additional costs; CR400.7h tracks the permitted spell's new stack incarnation. The opponent-active-seat test is explicitly a generic frame contract with an already-created real attack permission and an active-seat fixture change: it does not claim a Bilbo attack naturally occurs during an opponent's turn.

## Shared mechanics and wire

Private `CostTurn::NonHand` extends the existing typed reduction descriptor. Cost calculation reads captured cast origin from current/suspended announcements or stack; pre-announcement queries read the object's actual origin zone. A foreign-owned hand is not the caster's hand. Existing reduction availability/controller/layers and cost locking are reused; sacrificing Bilbo during a mana payment cannot revoke the locked discount.

`Instruction::CastGraveyard` opens the existing owned resolution casting frame with one-successful-cast quota. Public `ResolutionFreeCasts.paid: Option<ResolutionPaidCast>` is omitted for all previous frame producers. A paid frame exposes `paid: {kinds:28, exile_instants_sorceries:true}`. Its selected face uses ordinary Normal casting, permits X and ordinary cost choices, and does not waive additional costs. This is genuinely new public choice vocabulary: central163 is already adopted; root coordinates164 or later and inherited-row audits. This leaf changes no central schema file. Its public descriptor is re-exported from engine.rs; there is no new State field or action. Existing frame inline size is included through existing `size_of` accounting, and owned snapshot derives retain the descriptor. Existing producers gain only `paid:None`, preserving their serialization.

Normal paid chosen-face funding previews now pass the selected immutable definition explicitly to the funding helper; scalar-X permission previews use the existing definition-specific interval function. This avoids consulting a creature physical front for an Adventure sorcery's costs. No new timing keyword is granted.

## Obligations and assertions

| Obligation / boundary | Independent outcome |
| --- | --- |
| Paid body and hand exclusion | {1}{U} paid;2/2 Halfling Rogue; summoning-sick attack rejected; hand Pick still costs2 |
| Nonhand discount | Real graveyard, exile and top-library casts reduce generic1; colored costs and zero floor preserved; current controller matters |
| Optional non-targeting choice | Decline preserves graveyard; wrong owner/kinds and unaffordable cards absent; no target announcement |
| One paid spell per packet | Actual paid artifact/instant; successful cast consumes quota; Staff's two attack packets independently permit two casts |
| Spell-face qualification | Creature front rejected; actual {1}{B} Clap! Snap! costsB and amasses; proper chosen name and exile flag |
| Variable and alternate costs | Paid Mind Spring X3 costs2UU, draws3 privately; Stir pays B+sacrifice or3B with no sacrifice |
| Replacement | Instant resolves/countered/fizzles to exile; artifact resolves to battlefield or counters to graveyard, later death still graveyard; later spell incarnation has no inherited replacement |
| Source / exact identity | Source leaves, permission survives but discount disappears; stale selected card incarnation rejected |
| Costs / legality | Real Ring mana sacrifices vigilant attacking Bilbo after cost locked at1; underpayment rejects transactionally; casting prohibition respected |
| Owned continuation | Raw/index selection, parent disposal, restore, wrong actor/stale/invalid actions and equivalent final state at paid frame/payment roots |
| Failed cast | Actual foreign Aura entry invalidates casualty; exact graveyard card and quota restored, retained/reversed Ring mana branches preserve their own results |

Preflight verified admitted fixture names, complete flags, chosen-face costs and action order against existing Stir tests. In particular Clap! Snap! costs1B; Stir's optional4 is additional toB; Lotus Ring grants vigilance. Author-only registration `mod hob_bilbo;` is excluded; root regenerates. Committed rollback registration is the disjoint tail of `sos_mana_rollback.rs` using `#[path="hob_bilbo_rollback.rs"] mod bilbo_tests;`.

## Validation

First coherent focused run passed:58 executions,58 unique scenarios,0 failures/ignores. Groups: own Bilbo16 (15 main and1 real failed-cast rollback), Dawning5, caster8, Stir/Stone6, inherited second Adventure cohort9, mana rollback3, cast rollback6 and storm5. All eight consuming jobs returned0 with no timeout/orphan. Native execution reported0.74s combined; these are bounded assurance scenarios, not simulation-throughput or whole-set acceptance measurements.

Persistent receipt `<cache>/hob-bilbo/run-12ea2452d38e491da8365c10debaaa52/`: driver.py, outer.log (entire guard stdout/stderr from launch), results.json, resource.json and0–7 group logs. Original author session33853 naturally terminated0. Guard57.865611796s including shared-slot wait; first compile35.93s; grouped command elapsed37.426608342s; peak2592333824bytes within6442450944byte cap, one job/thread, all OOM counters0. There were no failed/canceled author runs. The initial cache-directory preparation was read-only under the sandbox and was completed with authorized filesystem escalation before launch; no consuming job was affected.

Source pins:5417 definitions and1608 Java blobs verified. Rustfmt and git diff whitespace checks pass. Required root full validator and two independent nonauthor reviews remain integration obligations. This profile is scoped to author assurance; default full-release acceptance is unchanged. Stable target `<cache>/hob-bilbo/target`; every run has a persistent driver, whole guard outer log and incrementally persisted results, immutable from launch until natural termination. No cancellation or active-output deletion.


Receipt SHA256:

- `driver.py`: `b318b0d1a840ec290afec5779f2b8b454c3311ba24256341c8ca6e8c3fe6bc4c`
- `results.json`: `3362e1483b4c51639854ee23bbae87d68b93fe1d5663e9e757a817bde5f18e33`
- `resource.json`: `e51da04aabd2529bc3500223cfe5be18f40e32dbe1119c7cfeb434e1a217a53a`
- `outer.log`: `c654f8780b762af2a426519459c015458a70e848d914ee9c2b44cfb221d453a2`

Authored Rust SHA256:

- `src/card_definitions/hob/bilbo_thief_in_the_night.rs`: `ba636a516bb0b006510cb25dd3b796dfafd16b586619786940573a2e04ec41da`
- `src/assurance/hob_bilbo.rs`: `2b9a4f0b35c2d95c203370c23d1db5eb3553e2ed706cfbedc1bcd87ce4bc8c18`
- `src/assurance/hob_bilbo_rollback.rs`: `0b8a4e333912d5ea7378131b524db5fb0f8c92813ed2c671a3b9c8462f18c376`

Implementation interval from isolated checkout creation to final evidence recording: 714.7s, one complete candidate card and paid-resolution shared core; this includes source review, authoring and queue/build/proof, and is not merged-card or simulation throughput.
