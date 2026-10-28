//! Bounded rules assurance. Card outcomes use ordinary checked actions; expensive
//! ownership/rollback contracts are checked once at representative decision roots.
use super::*;
pub(super) mod big_helpers;
mod cards;
mod contracts;
mod interactions;
mod regressions;
mod requested_earthbend;
mod requested_full_bore;
mod requested_ghost;
mod requested_hunters_talent;
mod requested_pyrrhic_strike;

fn scenario(p: Player) -> Game {
    let mut g = Game::new(Config {
        decks: [vec![Card::Forest; 7], vec![Card::Forest; 7]],
        seed: 20261023,
        starting_player: p,
        deck_rules: DeckRules::Freeform,
    })
    .unwrap();
    // Reset only the opening deck/hand. Fresh games have no registered sources.
    g.state.objects.clear();
    for player in &mut g.state.players {
        player.library.clear();
        player.cards_drawn_this_turn = 0;
        player.kept = true;
        player.mana = [8; 6];
    }
    g.state.pending = Pending::Priority;
    g.state.step = Step::Main1;
    g.state.turn = 3;
    g.state.active = p;
    g.state.priority = p;
    for owner in [Player::P0, Player::P1] {
        for _ in 0..8 {
            put(&mut g, owner, "Forest", Zone::Library);
        }
    }
    g
}

fn put(g: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap_or_else(|| panic!("unknown fixture {name}"));
    assert!(
        card.is_supported() || name == "Ghost Vacuum" || name == "Pyrrhic Strike",
        "unsupported fixture {name}"
    );
    let id = g.state.objects.len() as CardId;
    g.state.objects.push(Object {
        copy_kind: CopyKind::None,
        card,
        owner,
        controller: owner,
        zone,
        generation: 0,
        tapped: false,
        summoning_sick: false,
        flashback: false,
        cast_for_warp: false,
        damage: 0,
        damaged_this_turn: false,
        dealt_damage_this_turn: false,
        deathtouch_since_sba: false,
        growth: Stats::default(),
        gained_keywords: Keywords::NONE,
        counters: InitialCounters::default(),
        used_abilities: 0,
        class_level: 1,
        attached_to: None,
    });
    match zone {
        Zone::Library => g.state.players[owner.index()].library.push(id),
        Zone::Battlefield => g.register_battlefield_sources(id),
        Zone::Graveyard => g.register_graveyard_sources(id),
        _ => (),
    }
    id
}
fn act(g: &mut Game, action: Action) {
    g.apply(g.actor().expect("nonterminal scenario"), action.clone())
        .unwrap_or_else(|error| panic!("{action:?}: {error:?}; pending {:?}", g.state.pending));
}
fn choose(g: &mut Game, predicate: impl Fn(&Action) -> bool) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(predicate)
        .unwrap_or_else(|| panic!("missing choice in {:?}", g.state.pending));
    act(g, a);
}
fn pay(g: &mut Game) {
    choose(g, |a| matches!(a, Action::PayCosts { .. }));
}
fn cast(g: &mut Game, id: CardId, target: Option<Target>) {
    if let Some(action) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Cast{card,target:t,..}if *card==id&&*t==target))
    {
        act(g, action);
    } else {
        announce(g, id, &target.into_iter().collect::<Vec<_>>());
    }
}

fn announce(g: &mut Game, id: CardId, targets: &[Target]) {
    act(g, Action::AnnounceCast { card: id });
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}
fn settle(g: &mut Game) {
    for _ in 0..48 {
        if g.outcome().is_some()
            || (g.state.stack.is_empty()
                && g.state.resolving.is_none()
                && g.state.pending == Pending::Priority)
        {
            return;
        }
        assert_eq!(
            g.state.pending,
            Pending::Priority,
            "scenario needs an explicit choice"
        );
        act(g, Action::Pass);
    }
    panic!("48-action resolution budget exhausted");
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn stats(g: &Game, id: CardId) -> (i128, i128) {
    let s = g.stats(id);
    (s.power, s.toughness)
}

#[path = "../engine_sos_black_red_tests.rs"]
mod sos_black_red;
#[path = "../engine_sos_multicolor_tests.rs"]
mod sos_multicolor;

mod sos_white_blue;

mod sos_milled_play;
mod sos_multicolor_next;

mod sos_discard_followon;
mod sos_exile_search;
mod sos_hand_attack;
mod sos_moment;
mod sos_multicolor_more;
mod sos_nita;
mod sos_review_regressions;
mod sos_white_blue_final;

mod sos_applied;
mod sos_killian;
mod sos_white_blue_dragons;

mod sos_br_audit;
mod sos_life_x;
mod sos_original_multicolor_audit;
mod sos_storm;
mod sos_wu_audit;
mod sos_zaffai;

mod sos_cast_rollback;
mod sos_complete_games;

mod sos_evidence_funding;

mod sos_mana_rollback;

mod msh_a;
mod msh_b;
mod msh_c;
mod msh_contracts;
mod msh_d;

mod hob_amass;
mod hob_equipment;
mod hob_existing_mechanics;
mod hob_recruit;

mod hob_arkenstone;
mod hob_existing_two;
mod hob_gollum_abandoned;
mod hob_graveyard_origin;
mod hob_hone;
mod hob_recruit_amass_sagas;
mod hob_sting;
mod hob_storied;

mod hob_existing_three;

mod hob_bejeweled;
mod hob_fili;
mod hob_four_effects;
mod hob_trigger_multiplicity;
mod hob_warg_company;
mod hob_weavemaster;

mod hob_death_return;

mod hob_inherited_initial;

mod hob_cast_timing;

mod hob_inherited_draw;

mod hob_inherited_adventures;
mod hob_stir_stone;

mod hob_inherited_lands;

mod hob_equip_cost_history;
mod hob_inherited_adventures_two;
mod hob_plate_company;

mod hob_inherited_creatures;

mod hob_crooked_galion;

mod hob_inherited_spells;
mod hob_spark_letters;

mod hob_belladonna_queen;

mod hob_dancing;

mod hob_inherited_artifacts;

mod hob_mariners;

mod hob_inherited_combat;

mod hob_lonely_misty;

mod hob_inherited_remaining;

mod hob_wandering;

mod hob_beorn;

mod hob_black_arrow;

mod hob_master;

mod hob_roll;

mod hob_azog_rampager;

mod hob_desolation;
mod hob_riddles;

mod hob_vow_thorin;

mod hob_great_goblin;

mod hob_balin;

mod hob_celebrate;

mod hob_elven_passage;

mod hob_desert_worm;

mod hob_bolg;

mod hob_elrond;

mod hob_bard;
mod hob_bilbo;
mod hob_bilbo_luckwearer;
mod hob_complete_games;
mod hob_dain_ironfoot;
mod hob_down_valley;
mod hob_eagles_coming;
mod hob_eagles_rescue;
mod hob_gandalf_flameshape;
mod hob_getaway_barrel;
mod hob_gollum_riddle;
mod hob_head_hunt;
mod hob_inside_information;
mod hob_key_side_door;
mod hob_last_light;
mod hob_notary;
mod hob_old_fat_spider;
mod hob_orcrist;
mod hob_part_friendship;
mod hob_roads;
mod hob_settle_wreckage;
mod hob_silvan_reveler;
mod hob_smaug_wicked;
mod hob_supper_spiders;
mod hob_thorin_mountain_king;
mod hob_thranduil;
mod hob_thranduil_decree;
mod hob_tom_bert_william;

mod requested_sleight;

mod requested_liliana;

mod requested_edge_tests;

mod requested_haliya;
mod requested_mightform;

mod requested_essence;

mod requested_secret_identity;

mod requested_icetill;

mod requested_meltstrider_lki;

mod requested_sandman_checks;

mod requested_defend;

mod requested_curator;
mod requested_lunar;

mod requested_bitter_triumph;

mod requested_confidant;

mod requested_surrak;

#[cfg(test)]
mod requested_dreadmaw;

#[cfg(test)]
mod requested_amalia;

#[cfg(test)]
mod requested_wild_ride;

mod requested_elusive_otter;
mod requested_fra_prepare;

mod requested_case;

mod requested_esper;

mod requested_battle_foundation;
mod requested_callous;

#[path = "../engine_fra_blue_tests.rs"]
mod fra_blue;

#[path = "../engine_fra_charms_tests.rs"]
mod fra_charms;
