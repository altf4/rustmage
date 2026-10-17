//! Paid printed control consumers: private search, exact incarnations and owned replay.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::tests::spawn;
use super::*;

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
fn until_search(g: &mut Game) {
    for _ in 0..30 {
        if matches!(g.state.pending, Pending::EffectSearch { .. }) {
            return;
        }
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap()
        };
        checked(g, action);
    }
    panic!("no private search: {:?}", g.state.pending);
}
fn wish(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Wishclaw Talisman", Zone::Hand);
    g.state.players[p.index()].mana = [1, 0, 1, 0, 0, 0];
    cast(g, id, None, &[]);
    settle(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.counter_count(id, CounterKind::Wish), 3);
    assert!(!g.state.objects[id as usize].tapped);
    id
}
fn activate(g: &mut Game, p: Player, id: CardId) {
    g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::of(CounterKind::Wish, 1),
        },
    );
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            break;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, action);
    }
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert!(g.state.objects[id as usize].tapped);
    assert_eq!(g.counter_count(id, CounterKind::Wish), 2);
}
fn offering(g: &mut Game, p: Player, victim: CardId) -> CardId {
    let spell = named(g, p, "Harmless Offering", Zone::Hand);
    g.state.players[p.index()].mana = [2, 0, 0, 1, 0, 0];
    cast(
        g,
        spell,
        None,
        &[Target::Player(p.other()), Target::Card(victim)],
    );
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    spell
}

#[test]
fn wishclaw_paid_private_required_search_shuffle_and_fixed_opponent_transfer() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let chosen = named(&mut g, p, "Bear Cub", Zone::Library);
        let foreign = named(&mut g, p.other(), "Opt", Zone::Library);
        let hand = named(&mut g, p, "Opt", Zone::Hand);
        let id = wish(&mut g, p);
        g.state.players[p.other().index()].hexproof_until_end_turn = true;
        activate(&mut g, p, id);
        until_search(&mut g);
        let before_library = g.state.players[p.index()].library.clone();
        let mut expected = before_library.clone();
        expected.retain(|&x| x != chosen);
        let mut expected_rng = g.state.rng;
        expected_rng.shuffle(&mut expected);
        let hidden = g.observe(p.other());
        assert!(hidden.search_choice.is_none());
        assert!(hidden.actions.is_empty());
        assert!(hidden.objects.iter().all(|o| o.zone != Zone::Library));
        reject(&mut g, Action::FinishSearch);
        for card in [foreign, hand, u32::MAX] {
            reject(&mut g, Action::ChooseSearch { card });
        }
        checked(&mut g, Action::ChooseSearch { card: chosen });
        assert_eq!(g.state.players[p.index()].library, before_library);
        assert_eq!(g.object_view(id).controller, p);
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library, expected);
        assert_eq!(g.state.rng, expected_rng);
        assert_eq!(g.state.objects[chosen as usize].zone, Zone::Hand);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == chosen));
        assert_eq!(g.object_view(id).controller, p.other());
        assert_eq!(g.object_view(id).owner, p);
        assert_eq!(g.counter_count(id, CounterKind::Wish), 2);
        next_turn(&mut g);
        assert_eq!(g.object_view(id).controller, p.other());
    }
}

#[test]
fn wishclaw_source_change_or_reentry_does_not_change_search_controller_or_donate_new_object() {
    for p in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = setup(p);
            let chosen = named(&mut g, p, "Opt", Zone::Library);
            let id = wish(&mut g, p);
            activate(&mut g, p, id);
            let original = g.object_ref(id);
            if reenter {
                g.move_object(id, Zone::Exile);
                g.move_object(id, Zone::Battlefield);
                assert_ne!(g.object_ref(id), original);
                assert_eq!(g.counter_count(id, CounterKind::Wish), 3);
            } else {
                let item = g.activation_item(p, id, 0);
                g.gain_control(&item, Subject::Source, PlayerSet::Opponent);
            }
            until_search(&mut g);
            assert_eq!(g.actor(), Some(p));
            checked(&mut g, Action::ChooseSearch { card: chosen });
            checked(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(g.state.objects[chosen as usize].zone, Zone::Hand);
            assert_eq!(
                g.object_view(id).controller,
                if reenter { p } else { p.other() }
            );
        }
    }
}

#[test]
fn wishclaw_empty_library_still_transfers_and_wrong_turn_tap_counter_costs_reject_atomically() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = wish(&mut g, p);
        let announce = Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::of(CounterKind::Wish, 1),
        };
        g.state.active = p.other();
        reject(&mut g, announce.clone());
        g.state.active = p;
        g.state.objects[id as usize].tapped = true;
        reject(&mut g, announce.clone());
        g.state.objects[id as usize].tapped = false;
        g.set_counters(id, Counters::default());
        reject(&mut g, announce);
        g.set_counters(id, Counters::of(CounterKind::Wish, 3));
        for card in g.state.players[p.index()]
            .library
            .clone()
            .iter()
            .copied()
            .collect::<Vec<_>>()
        {
            g.move_object(card, Zone::Exile);
        }
        activate(&mut g, p, id);
        settle(&mut g);
        assert_eq!(g.object_view(id).controller, p.other());
        assert!(g.state.players[p.index()].library.is_empty());
    }
}

#[test]
fn harmless_offering_paid_any_permanent_ownership_sickness_and_lasting_control() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Bear Cub",
            "Forest",
            "Ravenous Amulet",
            "Doubling Season",
            "Vivien Reid",
        ] {
            let mut g = setup(p);
            let victim = named(&mut g, p, name, Zone::Battlefield);
            if name == "Vivien Reid" {
                g.set_counters(victim, Counters::of(CounterKind::Loyalty, 5));
            }
            let spell = offering(&mut g, p, victim);
            settle(&mut g);
            assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
            assert_eq!(g.object_view(victim).controller, p.other());
            assert_eq!(g.object_view(victim).owner, p);
            assert!(g.state.objects[victim as usize].summoning_sick);
            next_turn(&mut g);
            assert_eq!(g.object_view(victim).controller, p.other());
            g.move_object(victim, Zone::Graveyard);
            assert_eq!(g.object_view(victim).owner, p);
        }
    }
}

#[test]
fn harmless_offering_independent_target_legality_and_illegal_announcement_are_transactional() {
    for p in [Player::P0, Player::P1] {
        for invalidate in 0..4 {
            let mut g = setup(p);
            let victim = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let other = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = named(&mut g, p, "Harmless Offering", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(other),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            for _ in 0..5 {
                if g.state.announcing.is_none() {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                    .unwrap();
                checked(&mut g, a);
            }
            match invalidate {
                0 => g.state.players[p.other().index()].hexproof_until_end_turn = true,
                1 => {
                    g.move_object(victim, Zone::Exile);
                }
                2 => {
                    g.move_object(victim, Zone::Exile);
                    g.move_object(victim, Zone::Battlefield);
                }
                _ => {
                    let mut item = g.activation_item(p, victim, 0);
                    item.targets.push(g.bind_target(Target::Card(victim)));
                    g.gain_control(&item, Subject::Targets, PlayerSet::Opponent);
                }
            }
            settle(&mut g);
            if invalidate == 0 || invalidate == 2 {
                assert_eq!(g.object_view(victim).controller, p);
            }
            if invalidate == 1 {
                assert_eq!(g.state.objects[victim as usize].zone, Zone::Exile);
            }
        }
    }
}

#[test]
fn fixed_named_entry_counters_are_replaced_before_entry_and_reset_per_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let card = Card::from_name("Wishclaw Talisman").unwrap();
        assert_eq!(card.enters_counters(), Counters::of(CounterKind::Wish, 3));
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let id = spawn(&mut g, p, card, Zone::Exile);
        let movement = g.entry_move(id, Zone::Battlefield, false);
        g.stage_entries(vec![movement], None);
        g.advance_entries();
        for _ in 0..20 {
            if g.state.objects[id as usize].zone == Zone::Battlefield {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap();
            checked(&mut g, a);
        }
        assert_eq!(g.counter_count(id, CounterKind::Wish), 6);
        let old = g.object_ref(id);
        let snapshot = g.snapshot();
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        branch.move_object(id, Zone::Exile);
        assert_eq!(branch.counter_count(id, CounterKind::Wish), 0);
        assert_eq!(g.counter_count(id, CounterKind::Wish), 6);
        let movement = branch.entry_move(id, Zone::Battlefield, false);
        branch.stage_entries(vec![movement], None);
        branch.advance_entries();
        for _ in 0..20 {
            if branch.state.objects[id as usize].zone == Zone::Battlefield {
                break;
            }
            let a = branch
                .legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap();
            checked(&mut branch, a);
        }
        assert_ne!(branch.object_ref(id), old);
        assert_eq!(branch.counter_count(id, CounterKind::Wish), 6);
        branch.restore(&snapshot).unwrap();
        assert_eq!(branch.snapshot(), g.snapshot());
    }
}

#[test]
fn wishclaw_own_turn_allows_response_timing_and_absent_source_still_searches() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = wish(&mut g, p);
        let chosen = named(&mut g, p, "Bear Cub", Zone::Library);
        let opt = named(&mut g, p, "Opt", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        cast(&mut g, opt, None, &[]);
        assert_eq!(g.state.stack.len(), 1);
        activate(&mut g, p, id);
        assert_eq!(g.state.stack.len(), 2);
        g.move_object(id, Zone::Graveyard);
        until_search(&mut g);
        checked(&mut g, Action::ChooseSearch { card: chosen });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.state.objects[chosen as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        assert_eq!(g.counter_count(id, CounterKind::Wish), 0);
    }
}
