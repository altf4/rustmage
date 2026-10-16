use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 8] = [
    "Arahbo, the First Fang",
    "Arni, Humble Scribe",
    "Black Panther, Vanguard",
    "Genghis Frog",
    "Knightfisher",
    "Lathliss, Dragon Queen",
    "Pactdoll Terror",
    "Thoughtweft Lieutenant",
];
const BASE: [(i128, i128); 8] = [
    (2, 2),
    (3, 2),
    (4, 4),
    (1, 3),
    (4, 5),
    (6, 6),
    (3, 4),
    (2, 2),
];
const HELPERS: [&str; 8] = [
    "Helpful Hunter",
    "Bear Cub",
    "Agent of Atlas",
    "Mutant Town Musicians",
    "Healer's Hawk",
    "Shivan Dragon",
    "Goldvein Pick",
    "Goldmeadow Nomad",
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
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(f.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(p, revision, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn choose(g: &mut Game, target: Option<CardId>) {
    let actions = g.legal_actions();
    let a = actions
        .iter()
        .find(|a| matches!(a, Action::ChooseMode { mode: 0 }))
        .or_else(|| {
            target.and_then(|id| {
                actions.iter().find(|a| {
                    **a == Action::ChooseTarget {
                        target: Target::Card(id),
                    }
                })
            })
        })
        .or_else(|| {
            actions.iter().find(|a| {
                matches!(
                    a,
                    Action::Discard { .. }
                        | Action::ChooseTarget { .. }
                        | Action::FinishTargets
                        | Action::PutTrigger { .. }
                        | Action::PayCosts { .. }
                        | Action::ChooseSpellCost { .. }
                        | Action::Pass
                )
            })
        })
        .expect("fixture choice")
        .clone();
    replay(g, a);
}
fn settle(g: &mut Game, target: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..200 {
        if g.outcome().is_some() || g.state.stack.is_empty() && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, target);
    }
    panic!("unsettled");
}
fn full_cast(g: &mut Game, id: CardId, target: Option<CardId>) {
    replay(g, Action::AnnounceCast { card: id });
    while g.observe(g.actor().unwrap()).announcing.is_some() {
        choose(g, target);
    }
    settle(g, target);
    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
}
fn pt(g: &Game, id: CardId) -> (i128, i128) {
    let o = g.object_view(id);
    (o.power, o.toughness)
}
fn factory(subtype: &str) -> Card {
    Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == subtype))
        .unwrap()
}
fn tokens(g: &Game, p: Player, subtype: &str) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.owner == p
                && o.zone == Zone::Battlefield
                && o.card.token_info().is_some_and(|t| t.subtypes == subtype)
        })
        .count()
}
fn count_created(g: &Game, p: Player, i: usize) -> usize {
    tokens(
        g,
        p,
        ["Cat", "", "Soldier", "Mutagen", "Fish", "Dragon", "", ""][i],
    )
}
fn intrinsic(i: usize) -> Keywords {
    match i {
        3 => Keywords::TRAMPLE,
        4 | 5 => Keywords::FLYING,
        _ => Keywords::NONE,
    }
}
#[test]
fn all_eight_complete_bodies_cast_and_match_a_real_owned_entrant_for_both_seats() {
    for p in [Player::P0, Player::P1] {
        for i in 0..8 {
            let mut g = setup(p);
            let id = named(&mut g, p, NAMES[i], Zone::Hand);
            full_cast(&mut g, id, Some(id));
            assert_eq!(
                pt(&g, id),
                (
                    BASE[i].0 + i128::from(i == 7),
                    BASE[i].1 + i128::from(i == 7)
                )
            );
            assert_eq!(
                g.keywords(id),
                if i == 7 {
                    Keywords::TRAMPLE
                } else {
                    intrinsic(i)
                }
            );
            assert_eq!(count_created(&g, p, i), usize::from(i == 0 || i == 3));
            assert_eq!(g.state.players[p.index()].life, 20 + i128::from(i == 6));
            assert_eq!(
                g.state.players[p.other().index()].life,
                20 - i128::from(i == 6)
            );
            if i == 1 {
                g.state.objects[id as usize].tapped = true;
            }
            let helper = named(&mut g, p, HELPERS[i], Zone::Hand);
            full_cast(&mut g, helper, Some(id));
            if [0, 2, 3, 4, 5].contains(&i) {
                assert_eq!(
                    count_created(&g, p, i),
                    if i == 0 || i == 3 { 2 } else { 1 }
                );
            }
            if i == 1 {
                assert!(!g.object_view(id).tapped);
            }
            if i == 0 {
                assert_eq!(pt(&g, helper), (2, 2));
                assert_eq!(pt(&g, id), (2, 2));
            }
            if i == 6 {
                assert_eq!(g.state.players[p.index()].life, 22);
                assert_eq!(g.state.players[p.other().index()].life, 18);
            }
            if i == 7 {
                assert_eq!(pt(&g, id), (4, 4));
                assert!(g.keywords(id).contains(Keywords::TRAMPLE));
            }
        }
    }
}
#[test]
fn nontoken_filters_exclude_matching_tokens_while_unqualified_mutant_kithkin_and_artifact_include_them()
 {
    for p in [Player::P0, Player::P1] {
        for (i, subtype) in [
            (0, "Cat"),
            (1, "Mutant"),
            (2, "Hero"),
            (3, "Mutant"),
            (4, "Bird"),
            (5, "Dragon"),
            (6, "Treasure"),
            (7, "Kithkin"),
        ] {
            let mut g = setup(p);
            let id = named(&mut g, p, NAMES[i], Zone::Battlefield);
            g.state.objects[id as usize].tapped = true;
            let before = count_created(&g, p, i);
            g.create_tokens(p, factory(subtype), 2, false);
            settle(&mut g, Some(id));
            if [0, 2, 4, 5].contains(&i) {
                assert_eq!(
                    count_created(&g, p, i),
                    before + if i == 0 || i == 5 { 2 } else { 0 }
                );
            }
            if i == 1 {
                assert!(g.object_view(id).tapped);
            }
            if i == 3 {
                assert_eq!(tokens(&g, p, "Mutagen"), 2);
            }
            if i == 6 {
                assert_eq!(g.state.players[p.index()].life, 22);
                assert_eq!(g.state.players[p.other().index()].life, 18);
            }
            if i == 7 {
                assert_eq!(pt(&g, id), (4, 4));
            }
        }
    }
}
#[test]
fn opponent_entries_and_wrong_types_do_not_match_but_changeling_matches_every_subtype() {
    for p in [Player::P0, Player::P1] {
        for i in 0..8 {
            let mut g = setup(p);
            let id = named(&mut g, p, NAMES[i], Zone::Battlefield);
            g.state.objects[id as usize].tapped = true;
            let foreign = named(&mut g, p.other(), HELPERS[i], Zone::Hand);
            g.move_object(foreign, Zone::Battlefield);
            settle(&mut g, Some(id));
            assert_eq!(count_created(&g, p, i), 0);
            assert_eq!(g.state.players[0].life, 20);
            assert_eq!(g.state.players[1].life, 20);
            assert_eq!(pt(&g, id), BASE[i]);
            assert!(g.object_view(id).tapped);
            let land = named(&mut g, p, "Plains", Zone::Hand);
            g.move_object(land, Zone::Battlefield);
            settle(&mut g, Some(id));
            assert_eq!(count_created(&g, p, i), 0);
            let mascot = named(&mut g, p, "Three Tree Mascot", Zone::Hand);
            full_cast(&mut g, mascot, Some(id));
            if [0, 2, 3, 4, 5].contains(&i) {
                assert_eq!(count_created(&g, p, i), 1);
            }
            if i == 1 {
                assert!(!g.object_view(id).tapped);
            }
            if i == 6 {
                assert_eq!(g.state.players[p.index()].life, 21);
                assert_eq!(g.state.players[p.other().index()].life, 19);
            }
            if i == 7 {
                assert_eq!(pt(&g, id), (3, 3));
            }
        }
    }
}
#[test]
fn pactdoll_observes_noncreature_artifacts_and_artifact_creatures_once_each() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[6], Zone::Hand);
        full_cast(&mut g, id, None);
        let pick = named(&mut g, p, "Goldvein Pick", Zone::Hand);
        full_cast(&mut g, pick, None);
        let mascot = named(&mut g, p, "Three Tree Mascot", Zone::Hand);
        full_cast(&mut g, mascot, None);
        g.create_tokens(p, factory("Treasure"), 2, false);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].life, 25);
        assert_eq!(g.state.players[p.other().index()].life, 15);
    }
}
#[test]
fn simultaneous_sources_and_changeling_see_post_batch_sources_without_self_double_counting() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let selected = [0, 3, 4, 5, 6, 7];
        let ids: Vec<_> = selected
            .iter()
            .map(|i| named(&mut g, p, NAMES[*i], Zone::Hand))
            .collect();
        let mascot = named(&mut g, p, "Three Tree Mascot", Zone::Hand);
        let mut batch = ids.clone();
        batch.push(mascot);
        let events = g.move_objects_raw(&batch, Zone::Battlefield);
        g.finish_zone_events(&events, &[]);
        settle(&mut g, Some(ids[5]));
        assert_eq!(tokens(&g, p, "Cat"), 2);
        assert_eq!(tokens(&g, p, "Mutagen"), 2);
        assert_eq!(tokens(&g, p, "Fish"), 1);
        assert_eq!(tokens(&g, p, "Dragon"), 1);
        assert_eq!(g.state.players[p.index()].life, 24);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert_eq!(pt(&g, ids[5]), (4, 4));
    }
}
#[test]
fn black_panther_both_modes_preserve_nontoken_hero_filter_and_current_squad_membership() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[2], Zone::Hand);
        full_cast(&mut g, id, None);
        let hero = named(&mut g, p, "Agent of Atlas", Zone::Hand);
        full_cast(&mut g, hero, None);
        assert_eq!(tokens(&g, p, "Soldier"), 1);
        let second = named(&mut g, p, "Agent of Atlas", Zone::Hand);
        g.move_object(second, Zone::Battlefield);
        g.prepare_triggers();
        assert_eq!(g.observe(p).decision, "spell_mode");
        assert!(g.observe(p.other()).mode_choices.is_empty());
        assert!(g.observe(p.other()).actions.is_empty());
        replay(&mut g, Action::ChooseMode { mode: 1 });
        settle(&mut g, None);
        assert_eq!(pt(&g, id), (5, 5));
        assert_eq!(pt(&g, hero), (3, 3));
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        full_cast(&mut g, bear, None);
        assert_eq!(pt(&g, bear), (2, 2));
        assert_eq!(tokens(&g, p, "Soldier"), 1);
        g.finish_cleanup();
        assert_eq!(pt(&g, id), (4, 4));
        assert_eq!(pt(&g, hero), (2, 2));
    }
}
#[test]
fn arni_activation_owns_private_discard_and_nontoken_entry_untaps_for_a_second_use() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[1], Zone::Hand);
        full_cast(&mut g, id, None);
        let activation = Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, activation.clone());
        g.begin_turn();
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [20; 6];
        for use_number in 0..2 {
            let before = g.state.players[p.index()].library.len();
            replay(&mut g, activation.clone());
            while g.observe(p).announcing.is_some() {
                choose(&mut g, None);
            }
            for _ in 0..12 {
                if matches!(g.state.pending, Pending::EffectDiscard { .. }) {
                    break;
                }
                choose(&mut g, None);
            }
            let other = g.observe(p.other());
            assert!(other.effect_discards.is_empty() && other.actions.is_empty());
            assert_eq!(g.observe(p).decision, "effect_discard");
            settle(&mut g, None);
            assert_eq!(g.state.players[p.index()].library.len(), before - 1);
            assert!(g.object_view(id).tapped);
            if use_number == 0 {
                g.create_tokens(p, factory("Mutant"), 1, false);
                settle(&mut g, None);
                assert!(g.object_view(id).tapped);
                let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
                full_cast(&mut g, bear, None);
                assert!(!g.object_view(id).tapped);
            }
        }
    }
}
#[test]
fn lathliss_full_activation_boosts_current_owned_dragons_only_and_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[5], Zone::Hand);
        full_cast(&mut g, id, None);
        let dragon = named(&mut g, p, "Shivan Dragon", Zone::Hand);
        full_cast(&mut g, dragon, None);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        while g.observe(p).announcing.is_some() {
            choose(&mut g, None);
        }
        settle(&mut g, None);
        assert_eq!(pt(&g, id), (7, 6));
        assert_eq!(pt(&g, dragon), (6, 5));
        assert_eq!(pt(&g, bear), (2, 2));
        assert_eq!(pt(&g, enemy), (5, 5));
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.owner == p
                    && o.zone == Zone::Battlefield
                    && o.card.token_info().is_some_and(|t| t.subtypes == "Dragon")
            })
            .unwrap()
            .0 as CardId;
        assert_eq!(pt(&g, token), (6, 5));
        assert!(g.keywords(token).contains(Keywords::FLYING));
        g.finish_cleanup();
        assert_eq!(pt(&g, id), (6, 6));
        assert_eq!(pt(&g, token), (5, 5));
    }
}
#[test]
fn genghis_mutagen_has_complete_sorcery_activation_target_cost_and_counter_effect() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[3], Zone::Hand);
        full_cast(&mut g, id, None);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.owner == p
                    && o.zone == Zone::Battlefield
                    && o.card.token_info().is_some_and(|t| t.subtypes == "Mutagen")
            })
            .unwrap()
            .0 as CardId;
        let a = Action::AnnounceActivate {
            card: token,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.active = p.other();
        reject(&mut g, a.clone());
        g.state.active = p;
        g.state.step = Step::Upkeep;
        reject(&mut g, a.clone());
        g.state.step = Step::Main1;
        replay(&mut g, a);
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        while g.observe(p).announcing.is_some() {
            choose(&mut g, None);
        }
        assert_eq!(tokens(&g, p, "Mutagen"), 0);
        settle(&mut g, None);
        assert_eq!(pt(&g, id), (2, 4));
        assert_eq!(g.object_view(id).counters.plus_one, 1);
    }
}
#[test]
fn departed_source_triggers_keep_selected_targets_and_modes_but_untap_cannot_touch_reentry() {
    for p in [Player::P0, Player::P1] {
        for i in 0..8 {
            let mut g = setup(p);
            let id = named(&mut g, p, NAMES[i], Zone::Battlefield);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let helper = named(&mut g, p, HELPERS[i], Zone::Hand);
            g.move_object(helper, Zone::Battlefield);
            g.prepare_triggers();
            for _ in 0..20 {
                if g.state.pending == Pending::Priority {
                    break;
                }
                choose(&mut g, Some(bear));
            }
            let old = g.object_ref(id);
            g.move_object(id, Zone::Exile);
            if i == 1 {
                g.move_object(id, Zone::Battlefield);
                g.state.objects[id as usize].tapped = true;
            }
            settle(&mut g, Some(bear));
            if [0, 2, 3, 4, 5].contains(&i) {
                assert_eq!(count_created(&g, p, i), 1);
            }
            if i == 1 {
                assert_ne!(old, g.object_ref(id));
                assert!(g.object_view(id).tapped);
            }
            if i == 6 {
                assert_eq!(g.state.players[p.index()].life, 21);
                assert_eq!(g.state.players[p.other().index()].life, 19);
            }
            if i == 7 {
                assert_eq!(pt(&g, bear), (3, 3));
                assert!(g.keywords(bear).contains(Keywords::TRAMPLE));
            }
        }
    }
}
#[test]
fn thoughtweft_targets_owned_creatures_and_does_not_retarget_a_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[7], Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let mascot = named(&mut g, p, "Three Tree Mascot", Zone::Hand);
        g.move_object(mascot, Zone::Battlefield);
        g.prepare_triggers();
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        while g.state.pending != Pending::Priority {
            choose(&mut g, None);
        }
        g.move_object(bear, Zone::Exile);
        g.move_object(bear, Zone::Battlefield);
        settle(&mut g, Some(id));
        assert_eq!(pt(&g, bear), (2, 2));
        assert!(!g.keywords(bear).contains(Keywords::TRAMPLE));
        assert_eq!(pt(&g, id), (2, 2));
    }
}
#[test]
fn pactdoll_life_overflow_rolls_back_the_entire_real_resolution_and_revision() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, NAMES[6], Zone::Battlefield);
    g.state.players[0].life = i128::MAX;
    g.create_tokens(Player::P0, factory("Treasure"), 1, false);
    g.prepare_triggers();
    replay(&mut g, Action::Pass);
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    assert!(matches!(
        g.apply(p, Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
#[test]
fn filtered_entry_batches_preserve_apnap_and_each_controllers_trigger_order() {
    for active in [Player::P0, Player::P1] {
        let mut g = setup(active);
        let mut ids = Vec::new();
        let mut entrants = Vec::new();
        for p in [Player::P0, Player::P1] {
            ids.push(named(&mut g, p, NAMES[0], Zone::Battlefield));
            ids.push(named(&mut g, p, NAMES[4], Zone::Battlefield));
            entrants.push(named(&mut g, p, "Three Tree Mascot", Zone::Hand));
        }
        let events = g.move_objects_raw(&entrants, Zone::Battlefield);
        g.finish_zone_events(&events, &[]);
        g.prepare_triggers();
        assert_eq!(g.actor(), Some(active));
        assert_eq!(g.observe(active).waiting_triggers.len(), 4);
        for _ in 0..10 {
            if g.state.pending == Pending::Priority {
                break;
            }
            choose(&mut g, None);
        }
        assert_eq!(g.state.stack.len(), 4);
        assert!(g.state.stack[..2].iter().all(|s| s.controller == active));
        assert!(
            g.state.stack[2..]
                .iter()
                .all(|s| s.controller == active.other())
        );
        settle(&mut g, None);
        for p in [Player::P0, Player::P1] {
            assert_eq!(tokens(&g, p, "Cat"), 1);
            assert_eq!(tokens(&g, p, "Fish"), 1);
        }
    }
}
