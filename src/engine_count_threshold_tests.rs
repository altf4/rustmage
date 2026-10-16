use super::tests::{act, cast, scenario, spawn};
use super::*;
const NAMES: [&str; 9] = [
    "Basking Capybara",
    "Billowing Shriekmass",
    "Brightspear Zealot",
    "Cloudsculpt Technician",
    "Frilled Cave-Wurm",
    "Gaelicat",
    "Gigantoad",
    "Mind Drill Assailant",
    "Scorpion Sentinel",
];
const BASE: [(i128, i128); 9] = [
    (1, 3),
    (2, 3),
    (2, 4),
    (1, 4),
    (2, 5),
    (1, 3),
    (4, 4),
    (2, 5),
    (1, 4),
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn pt(g: &Game, id: CardId) -> (i128, i128) {
    let o = g.object_view(id);
    (o.power, o.toughness)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(f.snapshot(), e);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.outcome().is_some() || g.state.stack.is_empty() && g.state.pending == Pending::Priority
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::PutTrigger { .. } | Action::ArrangeLibrary { .. }
                )
            })
            .expect("fixture frame");
        replay(g, a);
    }
    panic!("unsettled")
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, a);
}
fn full_cast(g: &mut Game, id: CardId) {
    replay(g, Action::AnnounceCast { card: id });
    if let Some(a) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
    {
        replay(g, a);
    }
    pay(g);
    settle(g);
}
fn grave(g: &mut Game, p: Player, n: usize, name: &str) -> Vec<CardId> {
    (0..n).map(|_| named(g, p, name, Zone::Graveyard)).collect()
}
fn lands(g: &mut Game, p: Player, n: usize) -> Vec<CardId> {
    (0..n)
        .map(|i| {
            named(
                g,
                p,
                if i % 2 == 0 {
                    "Forest"
                } else {
                    "Temple Garden"
                },
                Zone::Battlefield,
            )
        })
        .collect()
}
fn token(g: &mut Game, p: Player, z: Zone) -> CardId {
    let c = Card::catalogue()
        .find(|c| c.is_token() && c.is_artifact() && !c.is_creature())
        .unwrap();
    spawn(g, p, c, z)
}
#[test]
fn all_nine_bodies_really_cast_with_full_stats_keywords_mill_and_hybrid_costs() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Hand);
            let before = g.state.players[p.index()].library.len();
            full_cast(&mut g, id);
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_eq!(pt(&g, id), BASE[i], "{name}");
            let k = g.keywords(id);
            assert_eq!(k.contains(Keywords::FLYING), [1, 3, 5].contains(&i));
            assert_eq!(k.contains(Keywords::VIGILANCE), [2, 5].contains(&i));
            assert_eq!(
                g.state.players[p.index()].library.len(),
                before - if i == 1 { 3 } else { 0 }
            );
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
            assert_eq!(g.observe(p).schema, 51);
        }
    }
}
#[test]
fn artifact_thresholds_are_live_owned_zone_filtered_and_count_each_object_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cloud = named(&mut g, p, NAMES[3], Zone::Battlefield);
        let cat = named(&mut g, p, NAMES[5], Zone::Battlefield);
        let enemy = named(&mut g, p.other(), NAMES[3], Zone::Battlefield);
        named(&mut g, p.other(), "Scorpion Sentinel", Zone::Battlefield);
        named(&mut g, p, "Scorpion Sentinel", Zone::Hand);
        named(&mut g, p, "Scorpion Sentinel", Zone::Graveyard);
        assert_eq!(pt(&g, cloud), (1, 4));
        assert_eq!(pt(&g, cat), (1, 3));
        assert_eq!(pt(&g, enemy), (2, 4));
        let multi = named(&mut g, p, "Scorpion Sentinel", Zone::Battlefield);
        assert_eq!(pt(&g, cloud), (2, 4));
        assert_eq!(pt(&g, cat), (1, 3));
        let t = token(&mut g, p, Zone::Battlefield);
        assert_eq!(pt(&g, cat), (3, 3));
        let saved = g.snapshot();
        g.move_object(multi, Zone::Exile);
        assert_eq!(pt(&g, cat), (1, 3));
        assert_eq!(pt(&g, cloud), (2, 4));
        g.move_object(t, Zone::Graveyard);
        assert_eq!(pt(&g, cloud), (1, 4));
        g.restore(&saved).unwrap();
        assert_eq!(pt(&g, cat), (3, 3));
    }
}
#[test]
fn descend_counts_only_owned_permanent_cards_not_tokens_spells_or_types() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cap = named(&mut g, p, NAMES[0], Zone::Battlefield);
        let wurm = named(&mut g, p, NAMES[4], Zone::Battlefield);
        grave(&mut g, p, 8, "Opt");
        grave(&mut g, p.other(), 5, "Forest");
        token(&mut g, p, Zone::Graveyard);
        let ids = ["Forest", "Scorpion Sentinel", "Anthem of Champions"]
            .map(|n| named(&mut g, p, n, Zone::Graveyard));
        assert_eq!(pt(&g, cap), (1, 3));
        assert_eq!(pt(&g, wurm), (2, 5));
        let fourth = named(&mut g, p, "Scorpion Sentinel", Zone::Graveyard);
        assert_eq!(pt(&g, cap), (4, 3));
        assert_eq!(pt(&g, wurm), (4, 5));
        let s = g.snapshot();
        g.move_object(ids[0], Zone::Hand);
        assert_eq!(pt(&g, cap), (1, 3));
        assert_eq!(pt(&g, wurm), (2, 5));
        g.restore(&s).unwrap();
        assert_eq!(pt(&g, cap), (4, 3));
        g.move_object(fourth, Zone::Exile);
        assert_eq!(pt(&g, wurm), (2, 5));
    }
}
#[test]
fn threshold_includes_nonpermanent_cards_but_not_tokens_or_opponents_graves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let shriek = named(&mut g, p, NAMES[1], Zone::Battlefield);
        let mind = named(&mut g, p, NAMES[7], Zone::Battlefield);
        grave(&mut g, p, 6, "Opt");
        grave(&mut g, p.other(), 7, "Opt");
        token(&mut g, p, Zone::Graveyard);
        assert_eq!(pt(&g, shriek), (2, 3));
        assert_eq!(pt(&g, mind), (2, 5));
        let seventh = named(&mut g, p, "Forest", Zone::Graveyard);
        assert_eq!(pt(&g, shriek), (4, 4));
        assert_eq!(pt(&g, mind), (5, 5));
        let s = g.snapshot();
        g.move_object(seventh, Zone::Exile);
        assert_eq!(pt(&g, shriek), (2, 3));
        g.restore(&s).unwrap();
        assert_eq!(pt(&g, mind), (5, 5));
    }
}
#[test]
fn seven_land_bonuses_count_basics_nonbasics_and_controller_but_never_scale_past_threshold() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let frog = named(&mut g, p, NAMES[6], Zone::Battlefield);
        let robot = named(&mut g, p, NAMES[8], Zone::Battlefield);
        let ids = lands(&mut g, p, 6);
        lands(&mut g, p.other(), 9);
        named(&mut g, p, "Forest", Zone::Hand);
        named(&mut g, p, "Forest", Zone::Graveyard);
        assert_eq!(pt(&g, frog), (4, 4));
        assert_eq!(pt(&g, robot), (1, 4));
        lands(&mut g, p, 1);
        assert_eq!(pt(&g, frog), (6, 6));
        assert_eq!(pt(&g, robot), (4, 4));
        lands(&mut g, p, 10);
        assert_eq!(pt(&g, frog), (6, 6));
        assert_eq!(pt(&g, robot), (4, 4));
        for id in ids {
            g.move_object(id, Zone::Hand);
        }
        assert_eq!(pt(&g, frog), (6, 6));
    }
}
#[test]
fn shriekmass_real_entry_mills_three_and_crosses_seven_without_requiring_permanents() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grave(&mut g, p, 4, "Opt");
        let id = named(&mut g, p, NAMES[1], Zone::Hand);
        full_cast(&mut g, id);
        assert_eq!(pt(&g, id), (4, 4));
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.owner == p && o.zone == Zone::Graveyard)
                .count(),
            7
        );
        assert!(g.keywords(id).contains(Keywords::FLYING));
    }
}
#[test]
fn losing_threshold_toughness_causes_damage_based_death_in_the_shared_sba() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ids = grave(&mut g, p, 7, "Opt");
        let id = named(&mut g, p, NAMES[1], Zone::Battlefield);
        g.state.objects[id as usize].damage = 3;
        g.state_based_actions();
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        let s = g.snapshot();
        g.move_object(ids[0], Zone::Exile);
        assert_eq!(pt(&g, id), (2, 3));
        g.state_based_actions();
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.state.creatures_died_this_turn, 1);
        assert_eq!(pt(&g, id), (2, 3));
        g.restore(&s).unwrap();
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert_eq!(pt(&g, id), (4, 4));
    }
}
#[test]
fn mind_drill_hybrid_activation_pays_before_private_surveil_and_choice_changes_threshold() {
    for p in [Player::P0, Player::P1] {
        for color in [1, 2] {
            for bury in [false, true] {
                let mut g = setup(p);
                grave(&mut g, p, 6, "Opt");
                let id = named(&mut g, p, NAMES[7], Zone::Battlefield);
                g.state.players[p.index()].mana = [0; 6];
                g.state.players[p.index()].mana[color] = 3;
                replay(
                    &mut g,
                    Action::AnnounceActivate {
                        card: id,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::ChooseActivationCost { .. }))
                    .unwrap();
                replay(&mut g, a);
                reject(&mut g, Action::PayCosts { payment: [0; 6] });
                pay(&mut g);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                replay(&mut g, Action::Pass);
                replay(&mut g, Action::Pass);
                let own = g.observe(p);
                assert_eq!(own.library_choice.unwrap().remaining.len(), 1);
                assert!(g.observe(p.other()).library_choice.is_none());
                assert!(g.observe(p.other()).actions.is_empty());
                let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ArrangeLibrary{destination,..}if *destination==if bury{LibraryDestination::Graveyard}else{LibraryDestination::Top})).unwrap();
                replay(&mut g, a);
                settle(&mut g);
                assert_eq!(pt(&g, id), if bury { (5, 5) } else { (2, 5) });
                assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
            }
        }
    }
}
#[test]
fn zealot_counts_cast_commitment_before_resolution_including_countered_spells_and_own_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let z = named(&mut g, p, NAMES[2], Zone::Hand);
        full_cast(&mut g, z);
        assert_eq!(pt(&g, z), (2, 4));
        let opt = named(&mut g, p, "Opt", Zone::Hand);
        cast(&mut g, opt, None);
        assert_eq!(pt(&g, z), (4, 4));
        replay(&mut g, Action::Pass);
        let counter = named(&mut g, p.other(), "Counterspell", Zone::Hand);
        let target = Target::Spell(opt);
        cast(&mut g, counter, Some(target));
        settle(&mut g);
        assert_eq!(g.object_view(opt).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2);
        assert_eq!(g.state.players[p.other().index()].spells_cast_this_turn, 1);
        assert_eq!(pt(&g, z), (4, 4));
        assert!(g.keywords(z).contains(Keywords::VIGILANCE));
    }
}
#[test]
fn zealot_uses_earlier_casts_before_it_exists_and_both_histories_reset_on_actual_next_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for _ in 0..2 {
            let opt = named(&mut g, p, "Opt", Zone::Hand);
            cast(&mut g, opt, None);
            settle(&mut g);
        }
        let z = named(&mut g, p, NAMES[2], Zone::Hand);
        full_cast(&mut g, z);
        assert_eq!(pt(&g, z), (4, 4));
        let turn = g.state.turn;
        for _ in 0..120 {
            if g.state.turn > turn {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::FinishAttackers
                            | Action::FinishBlockers
                            | Action::Discard { .. }
                    )
                })
                .unwrap();
            replay(&mut g, a);
        }
        assert!(g.state.turn > turn);
        assert_eq!(
            g.state.players.each_ref().map(|p| p.spells_cast_this_turn),
            [0, 0]
        );
        assert_eq!(pt(&g, z), (2, 4));
    }
}
#[test]
fn source_bonuses_are_battlefield_only_stack_with_existing_layers_and_observe_purely() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        lands(&mut g, p, 7);
        let id = named(&mut g, p, NAMES[6], Zone::Battlefield);
        let lord = named(&mut g, p, "Anthem of Champions", Zone::Battlefield);
        g.state.objects[id as usize].counters.plus_one = 2;
        g.state.objects[id as usize].growth = Stats {
            power: 3,
            toughness: 1,
        };
        assert_eq!(pt(&g, id), (12, 10));
        let s = g.snapshot();
        let r = g.revision();
        for _ in 0..3 {
            g.observe(p);
            g.observe(p.other());
            g.legal_actions();
        }
        assert_eq!(g.snapshot(), s);
        assert_eq!(g.revision(), r);
        g.move_object(lord, Zone::Hand);
        assert_eq!(pt(&g, id), (11, 9));
        for zone in [
            Zone::Hand,
            Zone::Graveyard,
            Zone::Exile,
            Zone::Stack,
            Zone::Library,
        ] {
            g.move_object(id, zone);
            assert_eq!(pt(&g, id), (4, 4));
            assert!(!g.state.static_sources.contains(&id));
        }
        g.restore(&s).unwrap();
        assert_eq!(pt(&g, id), (12, 10));
        assert_eq!(
            g.state
                .static_sources
                .iter()
                .filter(|&&source| source == id)
                .count(),
            1
        );
    }
}
#[test]
fn actual_combat_uses_the_current_land_threshold_power_for_both_seats() {
    for p in [Player::P0, Player::P1] {
        for enabled in [false, true] {
            let mut g = setup(p);
            lands(&mut g, p, if enabled { 7 } else { 6 });
            let id = named(&mut g, p, NAMES[6], Zone::Battlefield);
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            replay(&mut g, Action::Attack { card: id });
            replay(&mut g, Action::FinishAttackers);
            for _ in 0..80 {
                if g.state.players[p.other().index()].life < 20 {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::FinishBlockers))
                    .unwrap();
                replay(&mut g, a);
            }
            assert_eq!(
                g.state.players[p.other().index()].life,
                if enabled { 14 } else { 16 }
            );
        }
    }
}
#[test]
fn maximal_spell_history_keeps_bonus_boolean_and_failed_cast_is_fully_atomic() {
    let p = Player::P0;
    let mut g = setup(p);
    let id = named(&mut g, p, NAMES[2], Zone::Battlefield);
    g.state.players[0].spells_cast_this_turn = u64::MAX;
    assert_eq!(pt(&g, id), (4, 4));
    let spell = named(&mut g, p, "Opt", Zone::Hand);
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Cast{card,..}if *card==spell))
        .unwrap();
    let s = g.snapshot();
    let r = g.revision();
    for _ in 0..2 {
        assert!(matches!(
            g.apply(p, a.clone()),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), s);
        assert_eq!(g.revision(), r);
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
        assert_eq!(pt(&g, id), (4, 4));
    }
}
