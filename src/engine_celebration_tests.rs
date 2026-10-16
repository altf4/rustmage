use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 10] = [
    "Armory Mice",
    "Ash, Party Crasher",
    "Belligerent of the Ball",
    "Bespoke Battlegarb",
    "Gallant Pie-Wielder",
    "Grand Ball Guest",
    "Lady of Laughter",
    "Pests of Honor",
    "Raging Battle Mouse",
    "Tuinvale Guide",
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
    g.state.players[p.index()].mana = [30; 6];
    g
}
fn enter(g: &mut Game, p: Player, n: &str) -> CardId {
    let id = named(g, p, n, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    id
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(f.snapshot(), expected);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected)
}
fn next(g: &mut Game, target: Option<CardId>) {
    let aa = g.legal_actions();
    let a = target
        .and_then(|id| {
            aa.iter().find(|a| {
                **a == Action::ChooseTarget {
                    target: Target::Card(id),
                }
            })
        })
        .or_else(|| {
            aa.iter().find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::ChooseTarget { .. }
                        | Action::Pass
                )
            })
        })
        .unwrap_or_else(|| panic!("{:?} {aa:?}", g.state.pending))
        .clone();
    replay(g, a)
}
fn settle(g: &mut Game, target: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
        {
            return;
        }
        next(g, target)
    }
    panic!("unsettled")
}
fn paid(g: &mut Game, id: CardId) {
    replay(g, Action::AnnounceCast { card: id });
    while g.state.announcing.is_some() {
        next(g, None)
    }
    settle(g, None);
    assert_eq!(g.object_view(id).zone, Zone::Battlefield)
}
fn count(g: &Game, p: Player) -> u64 {
    g.observe(p).players[p.index()].nonland_permanents_entered_this_turn
}
#[test]
fn celebration_all_ten_paid_bodies_both_controllers() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Hand);
            paid(&mut g, id);
            assert_eq!(count(&g, p), 1);
            let other = enter(&mut g, p, "Bear Cub");
            assert_eq!(count(&g, p), 2);
            let v = g.object_view(id);
            match i {
                0 => assert_eq!((v.power, v.toughness), (3, 3)),
                4 => assert!(
                    v.keywords
                        .contains(Keywords::FIRST_STRIKE.union(Keywords::DOUBLE_STRIKE))
                ),
                5 => {
                    assert_eq!((v.power, v.toughness), (3, 3));
                    assert!(v.keywords.contains(Keywords::TRAMPLE))
                }
                9 => {
                    assert_eq!((v.power, v.toughness), (3, 3));
                    assert!(
                        v.keywords
                            .contains(Keywords::FLYING.union(Keywords::LIFELINK))
                    )
                }
                1 => {
                    assert!(v.keywords.contains(Keywords::HASTE));
                    g.state.step = Step::BeginCombat;
                    replay(&mut g, Action::Pass);
                    replay(&mut g, Action::Pass);
                    replay(&mut g, Action::Attack { card: id });
                    replay(&mut g, Action::FinishAttackers);
                    settle(&mut g, None);
                    assert_eq!(g.object_view(id).counters.plus_one, 1)
                }
                _ => {
                    g.begin_step(if i == 6 { Step::End } else { Step::BeginCombat });
                    settle(&mut g, Some(other));
                    match i {
                        2 => {
                            assert_eq!(g.object_view(other).power, 3);
                            assert!(g.keywords(other).contains(Keywords::MENACE))
                        }
                        3 => {
                            assert_eq!(g.object_view(id).attached_to, Some(g.object_ref(other)));
                            assert_eq!(g.object_view(other).power, 4)
                        }
                        6 => assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1),
                        7 => assert_eq!(g.object_view(id).counters.plus_one, 1),
                        8 => assert_eq!(
                            (g.object_view(other).power, g.object_view(other).toughness),
                            (3, 3)
                        ),
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}
#[test]
fn celebration_history_counts_tokens_reentries_and_never_departures_or_lands() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        assert_eq!(count(&g, p), 0);
        enter(&mut g, p, "Forest");
        enter(&mut g, p, "Mountain");
        let mice = enter(&mut g, p, "Armory Mice");
        assert_eq!(count(&g, p), 1);
        assert_eq!(g.stats(mice).toughness, 1);
        let foe = enter(&mut g, p.other(), "Bear Cub");
        assert_eq!(count(&g, p), 1);
        assert_eq!(count(&g, p.other()), 1);
        g.move_object(foe, Zone::Graveyard);
        assert_eq!(count(&g, p.other()), 1);
        let map = Card::catalogue()
            .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Map"))
            .unwrap();
        g.create_tokens(p, map, 2, false);
        assert_eq!(count(&g, p), 3);
        assert_eq!(g.stats(mice).toughness, 3);
        let tokens = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.owner == p && o.card == map && o.zone == Zone::Battlefield)
            .map(|(i, _)| i as CardId)
            .collect::<Vec<_>>();
        for id in tokens {
            g.move_object(id, Zone::Graveyard);
        }
        g.state_based_actions();
        assert_eq!(count(&g, p), 3);
        let old = g.object_ref(mice);
        g.move_object(mice, Zone::Exile);
        g.move_object(mice, Zone::Battlefield);
        assert_ne!(g.object_ref(mice), old);
        assert_eq!(count(&g, p), 4)
    }
}
#[test]
fn celebration_source_as_second_entry_and_late_source_see_full_history() {
    for p in [Player::P0, Player::P1] {
        for prior in [1, 2] {
            let mut g = setup(p);
            for _ in 0..prior {
                let id = enter(&mut g, p, "Bear Cub");
                g.move_object(id, Zone::Graveyard);
            }
            let source = enter(&mut g, p, "Grand Ball Guest");
            assert_eq!(count(&g, p), prior + 1);
            assert_eq!(g.stats(source).power, 3);
            assert!(g.keywords(source).contains(Keywords::TRAMPLE));
            let s = g.snapshot();
            for _ in 0..3 {
                g.legal_actions();
                g.observe(p);
                let _ = g.clone();
            }
            assert_eq!(g.snapshot(), s)
        }
    }
}
#[test]
fn celebration_intervening_if_checks_event_and_resolution_both_seats() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Belligerent of the Ball",
            "Lady of Laughter",
            "Pests of Honor",
            "Raging Battle Mouse",
            "Bespoke Battlegarb",
        ] {
            let mut g = setup(p);
            let id = enter(&mut g, p, name);
            let step = if name == "Lady of Laughter" {
                Step::End
            } else {
                Step::BeginCombat
            };
            g.begin_step(step);
            assert!(g.state.pending_triggers.is_empty());
            let bear = enter(&mut g, p, "Bear Cub");
            assert!(g.state.pending_triggers.is_empty());
            g.begin_step(step);
            assert_eq!(g.state.pending_triggers.len(), 1);
            g.prepare_triggers();
            while g.state.announcing.is_some() {
                next(&mut g, Some(bear))
            }
            let s = g.snapshot();
            g.state.players[p.index()].nonland_permanents_entered_this_turn = 0;
            settle(&mut g, Some(bear));
            assert_eq!(g.object_view(id).counters.plus_one, 0);
            assert_eq!(g.object_view(bear).power, 2);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            assert!(g.object_view(id).attached_to.is_none());
            g.restore(&s).unwrap();
            settle(&mut g, Some(bear));
            if name == "Pests of Honor" {
                assert_eq!(g.object_view(id).counters.plus_one, 1)
            } else if name == "Lady of Laughter" {
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1)
            } else {
                assert!(g.object_view(bear).power > 2)
            }
        }
    }
}
#[test]
fn celebration_begin_combat_is_own_turn_and_sources_may_leave() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = enter(&mut g, p, "Belligerent of the Ball");
        let target = enter(&mut g, p, "Bear Cub");
        g.state.active = p.other();
        g.begin_step(Step::BeginCombat);
        assert!(g.state.pending_triggers.is_empty());
        g.state.active = p;
        g.begin_step(Step::BeginCombat);
        g.prepare_triggers();
        while g.state.announcing.is_some() {
            next(&mut g, Some(target))
        }
        g.move_object(source, Zone::Graveyard);
        settle(&mut g, Some(target));
        assert_eq!(g.object_view(target).power, 3);
        assert!(g.keywords(target).contains(Keywords::MENACE));
        assert_eq!(count(&g, p), 2)
    }
}
#[test]
fn celebration_bespoke_optional_zero_and_equip_are_complete() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let eq = enter(&mut g, p, "Bespoke Battlegarb");
        let target = enter(&mut g, p, "Bear Cub");
        g.begin_step(Step::BeginCombat);
        settle(&mut g, None);
        assert!(g.object_view(eq).attached_to.is_none());
        g.state.step = Step::Main2;
        g.state.players[p.index()].mana = [0; 6];
        g.state.players[p.index()].mana[5] = 2;
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::AnnounceActivate{card,..} if *card==eq))
            .unwrap();
        replay(&mut g, a);
        while g.state.announcing.is_some() {
            next(&mut g, Some(target))
        }
        settle(&mut g, Some(target));
        assert_eq!(g.object_view(eq).attached_to, Some(g.object_ref(target)));
        assert_eq!(g.object_view(target).power, 4);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.step = Step::BeginCombat;
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..} if *card==eq))
        )
    }
}
#[test]
fn celebration_bespoke_source_reincarnation_and_target_reincarnation_do_not_attach() {
    for p in [Player::P0, Player::P1] {
        for source_leaves in [true, false] {
            let mut g = setup(p);
            let eq = enter(&mut g, p, "Bespoke Battlegarb");
            let target = enter(&mut g, p, "Bear Cub");
            g.begin_step(Step::BeginCombat);
            g.prepare_triggers();
            while g.state.announcing.is_some() {
                next(&mut g, Some(target))
            }
            let id = if source_leaves { eq } else { target };
            g.move_object(id, Zone::Exile);
            g.move_object(id, Zone::Battlefield);
            settle(&mut g, Some(target));
            assert!(g.object_view(eq).attached_to.is_none());
            assert_eq!(g.object_view(target).power, 2)
        }
    }
}
#[test]
fn celebration_history_survives_cleanup_priority_then_resets_both_players() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mice = enter(&mut g, p, "Armory Mice");
        let guide = enter(&mut g, p, "Miner's Guidewing");
        enter(&mut g, p.other(), "Bear Cub");
        g.state.objects[guide as usize].counters.minus_one = 1;
        g.state.objects[guide as usize].growth.toughness = 1;
        g.state.step = Step::Cleanup;
        let turn = g.state.turn;
        g.finish_cleanup();
        assert_eq!(g.state.turn, turn);
        assert_eq!(count(&g, p), 2);
        assert_eq!(g.stats(mice).toughness, 3);
        settle(&mut g, Some(mice));
        g.finish_cleanup();
        assert_eq!(g.state.turn, turn + 1);
        assert_eq!(count(&g, p), 0);
        assert_eq!(count(&g, p.other()), 0);
        assert_eq!(g.stats(mice).toughness, 1)
    }
}
#[test]
fn celebration_live_double_strike_and_lifelink_change_real_combat() {
    for p in [Player::P0, Player::P1] {
        for name in ["Gallant Pie-Wielder", "Tuinvale Guide"] {
            let mut g = setup(p);
            let attacker = enter(&mut g, p, name);
            enter(&mut g, p, "Bear Cub");
            g.state.attackers.push(Attacker {
                card: attacker,
                defender: Some(CombatDefender::Player(p.other())),
                blocked: false,
            });
            g.state.step = Step::Blockers;
            g.advance_step();
            if name == "Gallant Pie-Wielder" {
                assert_eq!(g.state.step, Step::FirstStrikeDamage);
                assert_eq!(g.state.players[p.other().index()].life, 18);
                replay(&mut g, Action::Pass);
                replay(&mut g, Action::Pass);
                assert_eq!(g.state.players[p.other().index()].life, 16)
            } else {
                assert_eq!(g.state.players[p.other().index()].life, 17);
                assert_eq!(g.state.players[p.index()].life, 23)
            }
        }
    }
}
#[test]
fn celebration_raging_second_spell_discount_exact_and_locked() {
    for p in [Player::P0, Player::P1] {
        for spells in [0, 1, 2] {
            let mut g = setup(p);
            let mouse = enter(&mut g, p, "Raging Battle Mouse");
            g.state.players[p.index()].spells_cast_this_turn = spells;
            let spell = named(&mut g, p, "Armory Mice", Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: spell });
            assert_eq!(
                g.observe(p).payment_request.unwrap().generic,
                if spells == 1 { 0 } else { 1 }
            );
            g.move_object(mouse, Zone::Graveyard);
            assert_eq!(
                g.observe(p).payment_request.unwrap().generic,
                if spells == 1 { 0 } else { 1 }
            );
            while g.state.announcing.is_some() {
                next(&mut g, None)
            }
            settle(&mut g, None)
        }
    }
}
#[test]
fn celebration_second_token_overflow_rolls_back_whole_batch_and_history() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spell = named(&mut g, p, "Dragon Fodder", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: spell });
        while g.state.announcing.is_some() {
            next(&mut g, None)
        }
        replay(&mut g, Action::Pass);
        g.state.players[p.index()].nonland_permanents_entered_this_turn = u64::MAX - 1;
        let s = g.snapshot();
        let r = g.revision();
        assert!(matches!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), s);
        assert_eq!(g.revision(), r)
    }
}
#[test]
fn celebration_initial_memory_and_public_history_are_owned() {
    let mut g = Game::new(Config {
        decks: [vec![Card::Forest; 60], vec![Card::Forest; 60]],
        seed: 1,
        starting_player: Player::P0,
        deck_rules: DeckRules::Freeform,
    })
    .unwrap();
    println!("celebration initial logical bytes {}", g.state_bytes());
    let s = g.snapshot();
    let p = Player::P0;
    enter(&mut g, p, "Bear Cub");
    let mut f = g.clone();
    enter(&mut f, p, "Armory Mice");
    assert_eq!(count(&g, p), 1);
    assert_eq!(count(&f, p), 2);
    assert_eq!(
        g.observe(p).players[0].nonland_permanents_entered_this_turn,
        g.observe(p.other()).players[0].nonland_permanents_entered_this_turn
    );
    g.restore(&s).unwrap();
    assert_eq!(count(&g, p), 0)
}
#[test]
fn celebration_explore_land_put_is_not_an_entry_and_existing_source_counts_before_observers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let scout = enter(&mut g, p, "Cenote Scout");
        settle(&mut g, None);
        assert_eq!(count(&g, p), 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert_eq!(g.state.players[p.index()].library.len(), 19);
        let factory = Card::catalogue()
            .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Map"))
            .unwrap();
        g.create_tokens(p, factory, 1, false);
        let map = (g.state.objects.len() - 1) as CardId;
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::AnnounceActivate{card,..} if *card==map))
            .unwrap();
        replay(&mut g, a);
        while g.state.announcing.is_some() {
            next(&mut g, Some(scout))
        }
        settle(&mut g, None);
        assert_eq!(count(&g, p), 2);
        assert_eq!(g.state.players[p.index()].library.len(), 18);
        assert_eq!(g.object_view(map).zone, Zone::Outside)
    }
}
#[test]
fn celebration_raging_actual_second_selected_x_spell_preserves_printed_mana_value_and_lock() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mouse = enter(&mut g, p, "Raging Battle Mouse");
        let first = named(&mut g, p, "Novice Inspector", Zone::Hand);
        paid(&mut g, first);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
        let spring = named(&mut g, p, "Mind Spring", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: spring });
        replay(&mut g, Action::ChooseX { value: 3 });
        let request = g.observe(p).payment_request.unwrap();
        assert_eq!(request.generic, 2);
        assert_eq!(request.required, [0, 2, 0, 0, 0, 0]);
        assert_eq!(g.observe(p).announcing.unwrap().mana_value(), 5);
        g.move_object(mouse, Zone::Graveyard);
        assert_eq!(g.observe(p).payment_request.unwrap().generic, 2);
        while g.state.announcing.is_some() {
            next(&mut g, None)
        }
        assert_eq!(g.state.stack.last().unwrap().x, Some(3));
        assert_eq!(g.state.stack.last().unwrap().mana_value(), 5);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3)
    }
}
