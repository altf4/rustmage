use super::tests::{act, scenario, spawn};
use super::*;
fn named(g: &mut Game, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, Player::P0, c, z)
}
fn setup() -> Game {
    let mut g = scenario();
    g.state.players[0].mana = [30; 6];
    g
}
fn food(g: &mut Game) -> CardId {
    let c = Card::catalogue()
        .filter(|c| c.is_token())
        .find(|c| c.token_info().unwrap().name == "Food Token")
        .unwrap();
    spawn(g, Player::P0, c, Zone::Battlefield)
}
fn grave(g: &mut Game, count: usize) -> Vec<CardId> {
    (0..count)
        .map(|_| named(g, "Forest", Zone::Graveyard))
        .collect()
}
fn replay(g: &mut Game, a: Action) {
    let p = g.actor().unwrap();
    let s = g.snapshot();
    let rev = g.revision();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let after = g.snapshot();
    assert_eq!(after, fork.snapshot());
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), after);
}
fn step(g: &mut Game) -> bool {
    let actions = g.legal_actions();
    let action = actions
        .iter()
        .find(|a| {
            matches!(
                a,
                Action::FinishTargets | Action::PayCosts { .. } | Action::PutTrigger { .. }
            )
        })
        .or_else(|| {
            actions
                .iter()
                .find(|a| matches!(a, Action::Pass | Action::DeclineForage))
        })
        .cloned();
    if let Some(a) = action {
        replay(g, a);
        true
    } else {
        false
    }
}
fn settle(g: &mut Game) {
    for _ in 0..100 {
        if g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.ordering_triggers.is_empty()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        assert!(step(g), "{:?}", g.observe(Player::P0));
    }
    panic!("not settled");
}
fn to_forage(g: &mut Game) {
    for _ in 0..100 {
        if matches!(g.state.pending, Pending::Forage { .. }) {
            return;
        }
        assert!(step(g), "{:?}", g.observe(Player::P0));
    }
    panic!("no forage");
}
fn pay(g: &mut Game, route: ForageRoute) {
    replay(g, Action::ChooseForageRoute { route });
    for _ in 0..3 {
        if !matches!(g.state.pending, Pending::Forage { .. }) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseForage { .. }))
            .unwrap();
        replay(g, a);
    }
}
#[test]
fn conditional_entries_both_routes_are_exact_optional_and_snapshot_owned() {
    for name in ["Bushy Bodyguard", "Treetop Sentries"] {
        for route in [ForageRoute::ExileThree, ForageRoute::Food] {
            let mut g = setup();
            let ids = grave(&mut g, 3);
            let f = food(&mut g);
            let id = named(&mut g, name, Zone::Hand);
            let before_hand = g
                .state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Hand)
                .count();
            replay(&mut g, Action::AnnounceCast { card: id });
            if name == "Bushy Bodyguard" {
                replay(&mut g, Action::ChooseOffspring { pay: false });
            }
            to_forage(&mut g);
            assert!(g.observe(Player::P0).forage.is_some());
            assert!(g.observe(Player::P1).forage.is_none());
            let before = g.snapshot();
            assert_eq!(
                g.apply(Player::P1, Action::DeclineForage),
                Err(Error::WrongActor)
            );
            assert_eq!(before, g.snapshot());
            pay(&mut g, route);
            settle(&mut g);
            if name == "Bushy Bodyguard" {
                assert_eq!(g.state.objects[id as usize].counters.plus_one, 2);
            } else {
                assert_eq!(
                    g.state
                        .objects
                        .iter()
                        .filter(|o| o.zone == Zone::Hand)
                        .count(),
                    before_hand
                );
            }
            for c in ids {
                assert_eq!(
                    g.state.objects[c as usize].zone,
                    if route == ForageRoute::ExileThree {
                        Zone::Exile
                    } else {
                        Zone::Graveyard
                    }
                );
            }
            assert_eq!(
                g.state.objects[f as usize].zone,
                if route == ForageRoute::Food {
                    Zone::Outside
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
#[test]
fn three_card_payment_never_partially_moves_or_reuses_a_card() {
    let mut g = setup();
    let ids = grave(&mut g, 3);
    let id = named(&mut g, "Treetop Sentries", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    to_forage(&mut g);
    replay(
        &mut g,
        Action::ChooseForageRoute {
            route: ForageRoute::ExileThree,
        },
    );
    let r = g.object_ref(ids[0]);
    replay(&mut g, Action::ChooseForage { object: r });
    assert!(
        ids.iter()
            .all(|&id| g.state.objects[id as usize].zone == Zone::Graveyard)
    );
    let s = g.snapshot();
    assert_eq!(
        g.apply(Player::P0, Action::ChooseForage { object: r }),
        Err(Error::IllegalAction)
    );
    assert_eq!(s, g.snapshot());
    replay(&mut g, Action::DeclineForage);
    settle(&mut g);
    assert!(
        ids.iter()
            .all(|&id| g.state.objects[id as usize].zone == Zone::Graveyard)
    );
}
#[test]
fn curious_reflexive_targets_are_chosen_only_after_payment_and_can_be_absent() {
    for count in [3, 4] {
        let mut g = setup();
        grave(&mut g, count);
        let id = named(&mut g, "Curious Forager", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        to_forage(&mut g);
        assert!(g.state.resolving.as_ref().unwrap().item.targets.is_empty());
        pay(&mut g, ForageRoute::ExileThree);
        if count == 3 {
            settle(&mut g);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Hand)
                    .count(),
                0
            );
        } else {
            for _ in 0..30 {
                if matches!(g.state.pending, Pending::Targets { .. }) {
                    break;
                }
                assert!(step(&mut g));
            }
            let targets = g.legal_actions();
            assert_eq!(
                targets
                    .iter()
                    .filter(|a| matches!(a, Action::ChooseTarget { .. }))
                    .count(),
                1
            );
            let a = targets
                .into_iter()
                .find(|a| matches!(a, Action::ChooseTarget { .. }))
                .unwrap();
            replay(&mut g, a);
            settle(&mut g);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Hand)
                    .count(),
                1
            );
        }
    }
}
#[test]
fn feed_alternative_declared_before_targets_adds_black_or_pays_forage_once() {
    for route in [
        ForageRoute::Black,
        ForageRoute::Food,
        ForageRoute::ExileThree,
    ] {
        let mut g = setup();
        grave(&mut g, 3);
        food(&mut g);
        let target = named(&mut g, "Grizzly Bears", Zone::Battlefield);
        let grow = named(&mut g, "Corpseberry Cultivator", Zone::Battlefield);
        let id = named(&mut g, "Feed the Cycle", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        assert!(matches!(g.state.pending, Pending::ForageMethod { .. }));
        replay(&mut g, Action::ChooseForageRoute { route });
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if route != ForageRoute::Black {
            assert!(matches!(g.state.pending, Pending::Forage { .. }));
            for _ in 0..3 {
                if !matches!(g.state.pending, Pending::Forage { .. }) {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::ChooseForage { .. }))
                    .unwrap();
                replay(&mut g, a);
            }
        }
        let view = g.observe(Player::P0).forage.unwrap();
        assert_eq!(view.route, Some(route));
        assert_eq!(
            view.selected.len(),
            match route {
                ForageRoute::Black => 0,
                ForageRoute::Food => 1,
                ForageRoute::ExileThree => 3,
            }
        );
        assert!(g.observe(Player::P1).forage.is_none());
        let request = g.observe(Player::P0).payment_request.unwrap();
        assert_eq!(
            request.required[2],
            if route == ForageRoute::Black { 2 } else { 1 }
        );
        settle(&mut g);
        assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
        assert_eq!(
            g.state.objects[grow as usize].counters.plus_one,
            if route == ForageRoute::Black { 0 } else { 1 }
        );
    }
}
#[test]
fn camellia_activation_qualifies_one_forage_and_food_trigger_before_its_growth() {
    for route in [ForageRoute::ExileThree, ForageRoute::Food] {
        let mut g = setup();
        grave(&mut g, 3);
        food(&mut g);
        let cam = named(&mut g, "Camellia, the Seedmiser", Zone::Battlefield);
        let grow = named(&mut g, "Corpseberry Cultivator", Zone::Battlefield);
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: cam,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        replay(&mut g, Action::ChooseForageRoute { route });
        for _ in 0..3 {
            if !matches!(g.state.pending, Pending::Forage { .. }) {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::ChooseForage { .. }))
                .unwrap();
            replay(&mut g, a);
        }
        settle(&mut g);
        assert_eq!(g.state.objects[cam as usize].counters.plus_one, 0);
        assert_eq!(g.state.objects[grow as usize].counters.plus_one, 2);
        let squirrels: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.card
                        .token_info()
                        .is_some_and(|t| t.name == "Squirrel Token")
            })
            .collect();
        assert_eq!(squirrels.len(), usize::from(route == ForageRoute::Food));
        for (id, o) in squirrels {
            assert_eq!(o.counters.plus_one, 1);
            assert!(g.keywords(id as CardId).contains(Keywords::MENACE));
        }
    }
}
#[test]
fn incidental_multiple_food_sacrifices_are_one_food_event_and_never_forage() {
    let mut g = setup();
    let cam = named(&mut g, "Camellia, the Seedmiser", Zone::Battlefield);
    let grow = named(&mut g, "Corpseberry Cultivator", Zone::Battlefield);
    let a = food(&mut g);
    let b = food(&mut g);
    g.sacrifice_batch(&[(Player::P0, a), (Player::P0, b)]);
    assert_eq!(g.state.pending_triggers.len(), 1);
    assert_eq!(g.state.pending_triggers[0].object, g.object_ref(cam));
    g.prepare_triggers();
    settle(&mut g);
    assert_eq!(g.state.objects[grow as usize].counters.plus_one, 0);
}
#[test]
fn unavailable_or_foreign_resources_offer_only_decline_without_consumption() {
    let mut g = setup();
    grave(&mut g, 2);
    let f = food(&mut g);
    g.state.objects[f as usize].owner = Player::P1;
    let id = named(&mut g, "Treetop Sentries", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    to_forage(&mut g);
    assert!(
        g.legal_actions()
            .iter()
            .all(|a| matches!(a, Action::Concede | Action::DeclineForage))
    );
    let s = g.snapshot();
    assert_eq!(
        g.apply(
            Player::P0,
            Action::ChooseForageRoute {
                route: ForageRoute::Food
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(s, g.snapshot());
    replay(&mut g, Action::DeclineForage);
    settle(&mut g);
}
#[test]
fn queued_entry_survives_source_departure_but_never_grows_new_incarnation() {
    let mut g = setup();
    grave(&mut g, 3);
    let id = named(&mut g, "Bushy Bodyguard", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    replay(&mut g, Action::ChooseOffspring { pay: false });
    to_forage(&mut g);
    let original = g.object_ref(id);
    g.move_object(id, Zone::Graveyard);
    g.move_object(id, Zone::Battlefield);
    assert_ne!(g.object_ref(id), original);
    pay(&mut g, ForageRoute::ExileThree);
    settle(&mut g);
    assert_eq!(g.state.objects[id as usize].counters.plus_one, 0);
}
#[test]
fn completion_overflow_restores_every_card_and_partial_selection() {
    let mut g = setup();
    let ids = grave(&mut g, 3);
    named(&mut g, "Corpseberry Cultivator", Zone::Battlefield);
    let id = named(&mut g, "Treetop Sentries", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    to_forage(&mut g);
    replay(
        &mut g,
        Action::ChooseForageRoute {
            route: ForageRoute::ExileThree,
        },
    );
    for &id in &ids[..2] {
        let object = g.object_ref(id);
        replay(&mut g, Action::ChooseForage { object });
    }
    g.state.next_stack_id = u64::MAX;
    let s = g.snapshot();
    let revision = g.revision();
    assert!(matches!(
        g.apply(
            Player::P0,
            Action::ChooseForage {
                object: g.object_ref(ids[2])
            }
        ),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), revision);
    assert!(
        ids.iter()
            .all(|&id| g.state.objects[id as usize].zone == Zone::Graveyard)
    );
}
#[test]
fn paid_bushy_parent_and_one_one_copy_offer_separate_entry_payments() {
    let mut g = setup();
    grave(&mut g, 6);
    let id = named(&mut g, "Bushy Bodyguard", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    replay(&mut g, Action::ChooseOffspring { pay: true });
    let mut payments = 0;
    for _ in 0..100 {
        if matches!(g.state.pending, Pending::Forage { .. }) {
            pay(&mut g, ForageRoute::ExileThree);
            payments += 1;
        } else if g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.ordering_triggers.is_empty()
            && g.state.pending_triggers.is_empty()
        {
            break;
        } else {
            assert!(step(&mut g));
        }
    }
    assert_eq!(payments, 2);
    assert_eq!(g.state.objects[id as usize].counters.plus_one, 2);
    let copy = g
        .state
        .objects
        .iter()
        .find(|o| {
            o.zone == Zone::Battlefield && o.card.is_token() && o.card.name() == "Bushy Bodyguard"
        })
        .unwrap();
    assert_eq!(copy.counters.plus_one, 2);
}
#[test]
fn curious_source_can_depart_and_reflexive_target_revalidates_exact_incarnation() {
    let mut g = setup();
    let ids = grave(&mut g, 4);
    let id = named(&mut g, "Curious Forager", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    to_forage(&mut g);
    g.move_object(id, Zone::Exile);
    pay(&mut g, ForageRoute::ExileThree);
    for _ in 0..20 {
        if matches!(g.state.pending, Pending::Targets { .. }) {
            break;
        }
        assert!(step(&mut g));
    }
    let remaining = ids
        .into_iter()
        .find(|&id| g.state.objects[id as usize].zone == Zone::Graveyard)
        .unwrap();
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(remaining),
        },
    );
    g.move_object(remaining, Zone::Exile);
    g.move_object(remaining, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(g.state.objects[remaining as usize].zone, Zone::Graveyard);
}
#[test]
fn feed_only_offers_fundable_methods_and_cannot_reuse_its_stack_card() {
    let mut g = setup();
    g.state.players[0].mana = [0, 0, 1, 0, 0, 1];
    grave(&mut g, 2);
    named(&mut g, "Grizzly Bears", Zone::Battlefield);
    let id = named(&mut g, "Feed the Cycle", Zone::Hand);
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: id })
    );
    let food = food(&mut g);
    assert!(
        g.legal_actions()
            .contains(&Action::AnnounceCast { card: id })
    );
    replay(&mut g, Action::AnnounceCast { card: id });
    let routes = g.observe(Player::P0).forage.unwrap().routes;
    assert_eq!(routes, vec![ForageRoute::Food]);
    assert!(!routes.contains(&ForageRoute::ExileThree));
    assert_eq!(g.state.objects[food as usize].zone, Zone::Battlefield);
}
#[test]
fn feed_black_surcharge_survives_generic_reduction_and_other_player_never_forages() {
    let mut g = setup();
    named(&mut g, "Stormcatch Mentor", Zone::Battlefield);
    let other = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Corpseberry Cultivator").unwrap(),
        Zone::Battlefield,
    );
    let target = named(&mut g, "Grizzly Bears", Zone::Battlefield);
    let id = named(&mut g, "Feed the Cycle", Zone::Hand);
    g.state.players[0].mana = [0, 0, 2, 0, 0, 0];
    replay(&mut g, Action::AnnounceCast { card: id });
    replay(
        &mut g,
        Action::ChooseForageRoute {
            route: ForageRoute::Black,
        },
    );
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    let request = g.observe(Player::P0).payment_request.unwrap();
    assert_eq!(request.generic, 0);
    assert_eq!(request.required, [0, 0, 2, 0, 0, 0]);
    settle(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(g.state.objects[other as usize].counters.plus_one, 0);
}
