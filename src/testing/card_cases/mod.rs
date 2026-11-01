use super::*;

fn scenario(p: Player) -> Game {
    let mut g = super::game();
    g.state.active = p;
    g.state.priority = p;
    g
}
fn cast(g: &mut Game, id: CardId, target: Option<Target>) {
    if let Some(action) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Cast {card,target:t,..} if *card==id && *t==target))
    {
        act(g, action);
    } else {
        super::cast(g, id, &target.into_iter().collect::<Vec<_>>());
    }
}
fn announce(g: &mut Game, id: CardId, targets: &[Target]) {
    super::cast(g, id, targets);
}
fn settle(g: &mut Game) {
    super::resolve(g);
}
fn reject(g: &mut Game, action: Action) {
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
}

#[path = "big_helpers.rs"]
mod big_helpers;

#[path = "fra_coverage_gaps.rs"]
mod fra_coverage_gaps;

#[path = "fra_way_loyalty_foundation.rs"]
mod fra_way_loyalty_foundation;

#[path = "hartford_artifact_activations.rs"]
mod hartford_artifact_activations;

#[path = "hartford_artifact_selection.rs"]
mod hartford_artifact_selection;

#[path = "hartford_blight.rs"]
mod hartford_blight;

#[path = "hartford_braided_net.rs"]
mod hartford_braided_net;

#[path = "hartford_craft_core.rs"]
mod hartford_craft_core;

#[path = "hartford_exile_play.rs"]
mod hartford_exile_play;

#[path = "hartford_grave_exile.rs"]
mod hartford_grave_exile;

#[path = "hartford_leatherhead.rs"]
mod hartford_leatherhead;

#[path = "hartford_library_selection.rs"]
mod hartford_library_selection;

#[path = "hartford_lively_dirge.rs"]
mod hartford_lively_dirge;

#[path = "hartford_mana_restrictions.rs"]
mod hartford_mana_restrictions;

#[path = "hartford_removal.rs"]
mod hartford_removal;

#[path = "hartford_returns.rs"]
mod hartford_returns;

#[path = "hartford_revelation.rs"]
mod hartford_revelation;

#[path = "hartford_riverchurn.rs"]
mod hartford_riverchurn;

#[path = "hartford_sawblades.rs"]
mod hartford_sawblades;

#[path = "hartford_speed.rs"]
mod hartford_speed;

#[path = "hartford_suppression.rs"]
mod hartford_suppression;

#[path = "hartford_transforming_faces.rs"]
mod hartford_transforming_faces;

#[path = "requested_aang.rs"]
mod requested_aang;

#[path = "requested_airbend_core.rs"]
mod requested_airbend_core;

#[path = "requested_bovine_intervention.rs"]
mod requested_bovine_intervention;

#[path = "requested_earthbend.rs"]
mod requested_earthbend;

#[path = "requested_floodpits_drowner.rs"]
mod requested_floodpits_drowner;

#[path = "requested_freebooter.rs"]
mod requested_freebooter;

#[path = "requested_full_bore.rs"]
mod requested_full_bore;

#[path = "requested_get_out.rs"]
mod requested_get_out;

#[path = "requested_ghost.rs"]
mod requested_ghost;

#[path = "requested_hunters_talent.rs"]
mod requested_hunters_talent;

#[path = "requested_inevitable_defeat.rs"]
mod requested_inevitable_defeat;

#[path = "requested_pyrrhic_strike.rs"]
mod requested_pyrrhic_strike;

#[path = "requested_reduction.rs"]
mod requested_reduction;

#[path = "requested_sunderflock.rs"]
mod requested_sunderflock;

#[path = "requested_waterbend.rs"]
mod requested_waterbend;

#[path = "engine_sos_black_red_tests.rs"]
mod sos_black_red;

#[path = "engine_sos_multicolor_tests.rs"]
mod sos_multicolor;

#[path = "sos_white_blue.rs"]
mod sos_white_blue;

#[path = "sos_milled_play.rs"]
mod sos_milled_play;

#[path = "sos_multicolor_next.rs"]
mod sos_multicolor_next;

#[path = "sos_discard_followon.rs"]
mod sos_discard_followon;

#[path = "sos_exile_search.rs"]
mod sos_exile_search;

#[path = "sos_hand_attack.rs"]
mod sos_hand_attack;

#[path = "sos_moment.rs"]
mod sos_moment;

#[path = "sos_multicolor_more.rs"]
mod sos_multicolor_more;

#[path = "sos_nita.rs"]
mod sos_nita;

#[path = "sos_white_blue_final.rs"]
mod sos_white_blue_final;

#[path = "sos_applied.rs"]
mod sos_applied;

#[path = "sos_killian.rs"]
mod sos_killian;

#[path = "sos_white_blue_dragons.rs"]
mod sos_white_blue_dragons;

#[path = "sos_br_audit.rs"]
mod sos_br_audit;

#[path = "sos_life_x.rs"]
mod sos_life_x;

#[path = "sos_original_multicolor_audit.rs"]
mod sos_original_multicolor_audit;

#[path = "sos_storm.rs"]
mod sos_storm;

#[path = "sos_wu_audit.rs"]
mod sos_wu_audit;

#[path = "sos_zaffai.rs"]
mod sos_zaffai;

#[path = "msh_a.rs"]
mod msh_a;

#[path = "msh_b.rs"]
mod msh_b;

#[path = "msh_c.rs"]
mod msh_c;

#[path = "msh_d.rs"]
mod msh_d;

#[path = "hob_amass.rs"]
mod hob_amass;

#[path = "hob_equipment.rs"]
mod hob_equipment;

#[path = "hob_existing_mechanics.rs"]
mod hob_existing_mechanics;

#[path = "hob_recruit.rs"]
mod hob_recruit;

#[path = "hob_arkenstone.rs"]
mod hob_arkenstone;

#[path = "hob_existing_two.rs"]
mod hob_existing_two;

#[path = "hob_gollum_abandoned.rs"]
mod hob_gollum_abandoned;

#[path = "hob_graveyard_origin.rs"]
mod hob_graveyard_origin;

#[path = "hob_hone.rs"]
mod hob_hone;

#[path = "hob_recruit_amass_sagas.rs"]
mod hob_recruit_amass_sagas;

#[path = "hob_sting.rs"]
mod hob_sting;

#[path = "hob_storied.rs"]
mod hob_storied;

#[path = "hob_existing_three.rs"]
mod hob_existing_three;

#[path = "hob_fili.rs"]
mod hob_fili;

#[path = "hob_four_effects.rs"]
mod hob_four_effects;

#[path = "hob_trigger_multiplicity.rs"]
mod hob_trigger_multiplicity;

#[path = "hob_warg_company.rs"]
mod hob_warg_company;

#[path = "hob_weavemaster.rs"]
mod hob_weavemaster;

#[path = "hob_death_return.rs"]
mod hob_death_return;

#[path = "hob_inherited_initial.rs"]
mod hob_inherited_initial;

#[path = "hob_cast_timing.rs"]
mod hob_cast_timing;

#[path = "hob_inherited_draw.rs"]
mod hob_inherited_draw;

#[path = "hob_inherited_adventures.rs"]
mod hob_inherited_adventures;

#[path = "hob_stir_stone.rs"]
mod hob_stir_stone;

#[path = "hob_inherited_lands.rs"]
mod hob_inherited_lands;

#[path = "hob_equip_cost_history.rs"]
mod hob_equip_cost_history;

#[path = "hob_inherited_adventures_two.rs"]
mod hob_inherited_adventures_two;

#[path = "hob_plate_company.rs"]
mod hob_plate_company;

#[path = "hob_inherited_creatures.rs"]
mod hob_inherited_creatures;

#[path = "hob_crooked_galion.rs"]
mod hob_crooked_galion;

#[path = "hob_inherited_spells.rs"]
mod hob_inherited_spells;

#[path = "hob_spark_letters.rs"]
mod hob_spark_letters;

#[path = "hob_belladonna_queen.rs"]
mod hob_belladonna_queen;

#[path = "hob_dancing.rs"]
mod hob_dancing;

#[path = "hob_inherited_artifacts.rs"]
mod hob_inherited_artifacts;

#[path = "hob_mariners.rs"]
mod hob_mariners;

#[path = "hob_inherited_combat.rs"]
mod hob_inherited_combat;

#[path = "hob_lonely_misty.rs"]
mod hob_lonely_misty;

#[path = "hob_inherited_remaining.rs"]
mod hob_inherited_remaining;

#[path = "hob_wandering.rs"]
mod hob_wandering;

#[path = "hob_beorn.rs"]
mod hob_beorn;

#[path = "hob_black_arrow.rs"]
mod hob_black_arrow;

#[path = "hob_master.rs"]
mod hob_master;

#[path = "hob_roll.rs"]
mod hob_roll;

#[path = "hob_azog_rampager.rs"]
mod hob_azog_rampager;

#[path = "hob_desolation.rs"]
mod hob_desolation;

#[path = "hob_riddles.rs"]
mod hob_riddles;

#[path = "hob_vow_thorin.rs"]
mod hob_vow_thorin;

#[path = "hob_great_goblin.rs"]
mod hob_great_goblin;

#[path = "hob_balin.rs"]
mod hob_balin;

#[path = "hob_celebrate.rs"]
mod hob_celebrate;

#[path = "hob_elven_passage.rs"]
mod hob_elven_passage;

#[path = "hob_desert_worm.rs"]
mod hob_desert_worm;

#[path = "hob_bolg.rs"]
mod hob_bolg;

#[path = "hob_bard.rs"]
mod hob_bard;

#[path = "hob_bilbo.rs"]
mod hob_bilbo;

#[path = "hob_bilbo_luckwearer.rs"]
mod hob_bilbo_luckwearer;

#[path = "hob_dain_ironfoot.rs"]
mod hob_dain_ironfoot;

#[path = "hob_down_valley.rs"]
mod hob_down_valley;

#[path = "hob_eagles_coming.rs"]
mod hob_eagles_coming;

#[path = "hob_eagles_rescue.rs"]
mod hob_eagles_rescue;

#[path = "hob_gandalf_flameshape.rs"]
mod hob_gandalf_flameshape;

#[path = "hob_getaway_barrel.rs"]
mod hob_getaway_barrel;

#[path = "hob_gollum_riddle.rs"]
mod hob_gollum_riddle;

#[path = "hob_head_hunt.rs"]
mod hob_head_hunt;

#[path = "hob_inside_information.rs"]
mod hob_inside_information;

#[path = "hob_key_side_door.rs"]
mod hob_key_side_door;

#[path = "hob_last_light.rs"]
mod hob_last_light;

#[path = "hob_old_fat_spider.rs"]
mod hob_old_fat_spider;

#[path = "hob_orcrist.rs"]
mod hob_orcrist;

#[path = "hob_part_friendship.rs"]
mod hob_part_friendship;

#[path = "hob_roads.rs"]
mod hob_roads;

#[path = "hob_settle_wreckage.rs"]
mod hob_settle_wreckage;

#[path = "hob_silvan_reveler.rs"]
mod hob_silvan_reveler;

#[path = "hob_supper_spiders.rs"]
mod hob_supper_spiders;

#[path = "hob_thorin_mountain_king.rs"]
mod hob_thorin_mountain_king;

#[path = "hob_thranduil.rs"]
mod hob_thranduil;

#[path = "hob_thranduil_decree.rs"]
mod hob_thranduil_decree;

#[path = "hob_tom_bert_william.rs"]
mod hob_tom_bert_william;

#[path = "requested_liliana.rs"]
mod requested_liliana;

#[path = "requested_haliya.rs"]
mod requested_haliya;

#[path = "requested_mightform.rs"]
mod requested_mightform;

#[path = "requested_essence.rs"]
mod requested_essence;

#[path = "requested_secret_identity.rs"]
mod requested_secret_identity;

#[path = "requested_sandman_checks.rs"]
mod requested_sandman_checks;

#[path = "requested_defend.rs"]
mod requested_defend;

#[path = "requested_curator.rs"]
mod requested_curator;

#[path = "requested_lunar.rs"]
mod requested_lunar;

#[path = "requested_bitter_triumph.rs"]
mod requested_bitter_triumph;

#[path = "requested_confidant.rs"]
mod requested_confidant;

#[path = "requested_surrak.rs"]
mod requested_surrak;

#[path = "requested_dreadmaw.rs"]
mod requested_dreadmaw;

#[path = "requested_amalia.rs"]
mod requested_amalia;

#[path = "requested_wild_ride.rs"]
mod requested_wild_ride;

#[path = "requested_elusive_otter.rs"]
mod requested_elusive_otter;

#[path = "requested_fra_prepare.rs"]
mod requested_fra_prepare;

#[path = "requested_case.rs"]
mod requested_case;

#[path = "requested_esper.rs"]
mod requested_esper;

#[path = "requested_battle_foundation.rs"]
mod requested_battle_foundation;

#[path = "requested_callous.rs"]
mod requested_callous;

#[path = "engine_fra_blue_tests.rs"]
mod fra_blue;

#[path = "engine_fra_charms_tests.rs"]
mod fra_charms;

#[path = "fra_soul_tether.rs"]
mod fra_soul_tether;

#[path = "fra_fatehold_empower.rs"]
mod fra_fatehold_empower;

#[path = "fra_prepared_control.rs"]
mod fra_prepared_control;

#[path = "fra_prepared_upkeep.rs"]
mod fra_prepared_upkeep;

#[path = "fra_rattler_dark.rs"]
mod fra_rattler_dark;

#[path = "fra_traxos_interactions.rs"]
mod fra_traxos_interactions;

#[path = "fra_barbs_garden.rs"]
mod fra_barbs_garden;

#[path = "fra_cruel_calculations.rs"]
mod fra_cruel_calculations;

#[path = "hartford_draw.rs"]
mod hartford_draw;

#[path = "hartford_discard.rs"]
mod hartford_discard;

#[path = "hartford_draw_classes.rs"]
mod hartford_draw_classes;

#[path = "engine_big_core_tests.rs"]
mod big_core_tests;

#[path = "engine_big_equipment_tests.rs"]
mod big_equipment_tests;

#[path = "engine_big_library_tests.rs"]
mod big_library_tests;

#[path = "engine_big_token_tests.rs"]
mod big_token_tests;

#[path = "engine_territory_forge_tests.rs"]
mod territory_forge_tests;

#[path = "engine_big_inherited_tests.rs"]
mod big_inherited_tests;

#[path = "engine_sos_green_colorless_tests.rs"]
mod sos_green_colorless_tests;

#[path = "engine_requested_sapling_tests.rs"]
mod engine_requested_sapling_tests;
