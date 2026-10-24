# Original SOS green, colorless and black-green audit

Baseline: `d5cd65c1`, membership reconciled by Oracle ID against the 105 already admitted SOS members. This ownership group contains **37** Oracle IDs: 29 green/colorless/land members (including canonical reprints) and eight black-green members. Full Oracle bodies in the pinned `data/standard/cards.json` and complete XMage card classes were read. Java source and the relevant helper hashes are pinned in each paired JSON. No behavior changes were needed after this audit.

Every scenario runs for both seats. Its paid casts/activations or legal land plays use the shared `checked` helper, which verifies wrong-actor atomicity, owned raw/index branch replay, parent disposal, restore and stale revisions. Tests additionally reject invalid targets/payments/choices and inspect hidden information. Keyword assembly is checked on paid permanents; shared combat rules supply the generic keyword behavior. Admission traces are not the evidence used for this audit.

## Per-Oracle evidence

| Oracle member | Oracle ID | Canonical definition / full XMage class | Helpers | Scenarios |
|---|---|---|---|---|
| Plains | `bc71ebf6-2056-41f7-be35-b2e5c34afa99` | `fra/plains.rs` / `Plains.java` | BasicLand | S1 |
| Island | `b2c6aa39-2d2a-459c-a555-fb48ba993373` | `fra/island.rs` / `Island.java` | BasicLand | S1 |
| Swamp | `56719f6a-1a6c-4c0a-8d21-18f7d7350b68` | `fra/swamp.rs` / `Swamp.java` | BasicLand | S1 |
| Mountain | `a3fb7228-e76b-4e96-a40e-20b5fed75685` | `fra/mountain.rs` / `Mountain.java` | BasicLand | S1 |
| Forest | `b34bb2dc-c1af-4d77-b0b3-a0fb342a5fc6` | `fra/forest.rs` / `Forest.java` | BasicLand | S1 |
| Aberrant Manawurm | `a0657586-0404-4bb0-a92d-25f040d560c1` | `sos/aberrant_manawurm.rs` / `AberrantManawurm.java` | ManaPaid | S10 |
| Deathcap Glade | `f6d24565-5b32-4eff-b2e0-6e2c25516ff0` | `sos/deathcap_glade.rs` / `DeathcapGlade.java` | EntryUnless | S1 |
| Dreamroot Cascade | `dd8538e6-cd5f-4a88-aff5-eb5e76ce8ddb` | `sos/dreamroot_cascade.rs` / `DreamrootCascade.java` | EntryUnless | S1 |
| Efflorescence | `181da74b-53fb-4e46-afc0-e7c990fda128` | `sos/efflorescence.rs` / `Efflorescence.java` | LifeCondition, LifeWatcher | S5 |
| Environmental Scientist | `f30aa897-7027-47e7-ae58-73cbc448c1f0` | `sos/environmental_scientist.rs` / `EnvironmentalScientist.java` | SearchHand | S3 |
| Fields of Strife | `a32e08fa-bea4-4ba9-a126-9bf0a91f67e2` | `sos/fields_of_strife.rs` / `FieldsOfStrife.java` | Surveil | S2 |
| Follow the Lumarets | `5bb38d08-582e-43aa-9507-7c20759978ec` | `sos/follow_the_lumarets.rs` / `FollowTheLumarets.java` | LifeCondition, LifeWatcher, LookPick | S6 |
| Forum of Amity | `349ea6c7-6b3e-417f-b082-b712e2b1635b` | `sos/forum_of_amity.rs` / `ForumOfAmity.java` | Surveil | S2 |
| Hungry Graffalon | `9494c499-6ead-491f-aeb0-6dbfac4d960c` | `sos/hungry_graffalon.rs` / `HungryGraffalon.java` | Increment | S7 |
| Mage Tower Referee | `3f27988b-0863-4cc1-8eb9-83bd0cd8561c` | `sos/mage_tower_referee.rs` / `MageTowerReferee.java` | Card class; existing typed primitives | S10 |
| Mindful Biomancer | `03af9d24-2df6-4318-82f2-8abdd22206f1` | `sos/mindful_biomancer.rs` / `MindfulBiomancer.java` | Limit | S9 |
| Noxious Newt | `1232c461-643c-40a3-901e-09b665f817da` | `sos/noxious_newt.rs` / `NoxiousNewt.java` | Card class; existing typed primitives | S9 |
| Paradox Gardens | `638ff242-63d5-457d-a7a6-40ad51052e2e` | `sos/paradox_gardens.rs` / `ParadoxGardens.java` | Surveil | S2 |
| Rancorous Archaic | `6a68fcaf-b1e8-451d-8e1c-b47db55c83cc` | `sos/rancorous_archaic.rs` / `RancorousArchaic.java` | Converge | S8 |
| Shattered Sanctum | `c854ecb0-cc60-4c48-a9aa-7f2348a7a8c6` | `sos/shattered_sanctum.rs` / `ShatteredSanctum.java` | EntryUnless | S1 |
| Shopkeeper's Bane | `e7572317-6bec-4fb0-81ac-76aec04f14ad` | `sos/shopkeeper_s_bane.rs` / `ShopkeepersBane.java` | Card class; existing typed primitives | S11 |
| Spectacle Summit | `33a4e73d-d93a-4b6f-88ff-cd53f20d178c` | `sos/spectacle_summit.rs` / `SpectacleSummit.java` | Surveil | S2 |
| Stormcarved Coast | `4722105b-0085-4bb8-bca1-9de0d3eb5600` | `sos/stormcarved_coast.rs` / `StormcarvedCoast.java` | EntryUnless | S1 |
| Strixhaven Skycoach | `e286a191-39d4-4073-b844-11d2aba911b4` | `sos/strixhaven_skycoach.rs` / `StrixhavenSkycoach.java` | SearchHand, Crew | S3, S4 |
| Sundown Pass | `5ad0b405-cca4-475e-985c-4d7e3599d87e` | `sos/sundown_pass.rs` / `SundownPass.java` | EntryUnless | S1 |
| Tenured Concocter | `0ecd4493-dfb4-4d56-a90e-08b74b474f39` | `sos/tenured_concocter.rs` / `TenuredConcocter.java` | LifeCondition, LifeWatcher | S11, S12 |
| Terramorphic Expanse | `1bd3e453-aa21-4ee6-95c2-d6d920ee8e7a` | `msc/terramorphic_expanse.rs` / `TerramorphicExpanse.java` | SearchBattlefield | S3 |
| Thornfist Striker | `d0d13d5d-677d-4e1a-9e58-2f9840581f6d` | `sos/thornfist_striker.rs` / `ThornfistStriker.java` | LifeCondition, LifeWatcher | S11, S12 |
| Titan's Grave | `595f0eb5-f521-4174-9c48-b89e85ea907c` | `sos/titan_s_grave.rs` / `TitansGrave.java` | Surveil | S2 |
| Bogwater Lumaret | `0109432d-5a2a-456f-ad39-b75cb4c73420` | `sos/bogwater_lumaret.rs` / `BogwaterLumaret.java` | Card class; existing typed primitives | S13 |
| Cauldron of Essence | `a2f8cde8-bf7b-4234-89f0-a95f9dc937e3` | `sos/cauldron_of_essence.rs` / `CauldronOfEssence.java` | Card class; existing typed primitives | S13, S15 |
| Grapple with Death | `dced5cab-1438-4264-b44c-54dc37ec73d3` | `sos/grapple_with_death.rs` / `GrappleWithDeath.java` | Card class; existing typed primitives | S18 |
| Old-Growth Educator | `2b7362a9-3518-4071-ab1b-40689a50e12a` | `sos/old_growth_educator.rs` / `OldGrowthEducator.java` | LifeCondition, LifeWatcher | S14 |
| Pest Mascot | `b8ecddf7-ba89-40dd-9f63-5bad27dd850d` | `sos/pest_mascot.rs` / `PestMascot.java` | LifeEvent | S13 |
| Professor Dellian Fel | `8c5fe985-69e1-4a03-99fc-a6fd2314a9a7` | `sos/professor_dellian_fel.rs` / `ProfessorDellianFel.java` | Emblem, LifeEvent, SavedLife | S17 |
| Teacher's Pest | `12397dbf-99a9-40c3-aa7c-76d09baad008` | `sos/teacher_s_pest.rs` / `TeachersPest.java` | Card class; existing typed primitives | S16 |
| Witherbloom, the Balancer | `fbb04a21-e513-4317-aac8-fa6df91c3438` | `sos/witherbloom_the_balancer.rs` / `WitherbloomTheBalancer.java` | Affinity, GrantedAffinity | S10 |

## Scenarios

All functions are in `src/engine_sos_green_colorless_tests.rs`.

- **S1**: `sos_original_basic_and_slow_land_play_thresholds_and_exact_colors`
- **S2**: `sos_original_surveil_land_paid_colors_tap_private_choices_and_departure`
- **S3**: `sos_original_scientist_skycoach_and_expanse_optional_private_basic_searches`
- **S4**: `sos_original_skycoach_crew_sick_creatures_exact_power_and_cleanup`
- **S5**: `sos_original_efflorescence_infusion_at_resolution_persistent_counters_and_fizzle`
- **S6**: `sos_original_follow_lumarets_infusion_private_quota_reveal_and_random_remainder`
- **S7**: `sos_original_graffalon_increment_strict_trigger_and_resolution_checks`
- **S8**: `sos_original_rancorous_converge_paid_colors_replacements_and_uncast_return`
- **S9**: `sos_original_biomancer_once_each_turn_and_newt_sickness_mana_deathtouch_fight`
- **S10**: `sos_original_manawurm_referee_and_witherbloom_distinguish_actual_payment_and_color`
- **S11**: `sos_original_shopkeeper_attack_life_and_live_infusion_control_and_removal`
- **S12**: `sos_original_tenured_opponent_target_optional_draw_and_thornfist_ward_payment`
- **S13**: `sos_original_bg_entry_death_life_events_and_mascot_count_events_not_amount`
- **S14**: `sos_original_old_growth_educator_checks_life_at_trigger_resolution`
- **S15**: `sos_original_cauldron_sorcery_exact_target_and_paid_sacrifice_drain`
- **S16**: `sos_original_teachers_pest_paid_grave_return_tapped_exact_incarnation_and_menace`
- **S17**: `sos_original_professor_dellian_all_loyalty_modes_and_source_independent_emblem`
- **S18**: `sos_original_grapple_artifact_or_creature_targets_and_whole_spell_fizzle`

## Helper source map

All paths are relative to the pinned XMage checkout; revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Paired definition JSON records the exact SHA-256.

- **BasicLand**: `Mage/src/main/java/mage/cards/basiclands/BasicLand.java`
- **EntryUnless**: `Mage/src/main/java/mage/abilities/common/EntersBattlefieldTappedUnlessAbility.java`
- **Surveil**: `Mage/src/main/java/mage/abilities/effects/keyword/SurveilEffect.java`
- **SearchHand**: `Mage/src/main/java/mage/abilities/effects/common/search/SearchLibraryPutInHandEffect.java`
- **SearchBattlefield**: `Mage/src/main/java/mage/abilities/effects/common/search/SearchLibraryPutInPlayEffect.java`
- **Crew**: `Mage/src/main/java/mage/abilities/keyword/CrewAbility.java`
- **LifeCondition**: `Mage/src/main/java/mage/abilities/condition/common/YouGainedLifeCondition.java`
- **LifeWatcher**: `Mage/src/main/java/mage/watchers/common/PlayerGainedLifeWatcher.java`
- **LookPick**: `Mage/src/main/java/mage/abilities/effects/common/LookLibraryAndPickControllerEffect.java`
- **Increment**: `Mage/src/main/java/mage/abilities/keyword/IncrementAbility.java`
- **Converge**: `Mage/src/main/java/mage/abilities/dynamicvalue/common/ColorsOfManaSpentToCastCount.java`
- **Limit**: `Mage/src/main/java/mage/abilities/common/LimitedTimesPerTurnActivatedAbility.java`
- **ManaPaid**: `Mage/src/main/java/mage/watchers/common/ManaPaidSourceWatcher.java`
- **LifeEvent**: `Mage/src/main/java/mage/abilities/common/GainLifeControllerTriggeredAbility.java`
- **SavedLife**: `Mage/src/main/java/mage/abilities/dynamicvalue/common/SavedGainedLifeValue.java`
- **Emblem**: `Mage/src/main/java/mage/game/command/emblems/ProfessorDellianFelEmblem.java`
- **Affinity**: `Mage/src/main/java/mage/abilities/keyword/AffinityAbility.java`
- **GrantedAffinity**: `Mage/src/main/java/mage/abilities/effects/common/continuous/GainAbilityControlledSpellsEffect.java`

## Validation

The focused green/colorless suite passes **89 tests** (including the 18 original-card groups) in 0.06 seconds. `scripts/check_sources.py` verifies all **895** immutable Java blobs and the offline land-name source derivation. Strict `cargo clippy --lib --tests -- -D warnings` passes with the already integrated BR baseline one-element-loop lint fix applied locally; that unrelated fix is excluded from this audit commit. All Cargo commands ran through `scripts/resource_guard.py`.
