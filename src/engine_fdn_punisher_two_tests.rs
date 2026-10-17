//! Complete paid punisher choices, with the shared owned/indexed action helper.
use super::ability_timeline_tests::{cast, checked, setup};
use super::tests::spawn;
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..180 {
        if stop(g) {
            return;
        }
        let legal = g.legal_actions();
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            legal
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. } | Action::FinishTargets | Action::PayCosts { .. }
                    )
                })
                .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending))
        };
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending)
}
fn settle(g: &mut Game) {
    until(g, idle);
}
fn choice(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(before, g.snapshot());
    assert_eq!(rev, g.revision());
}
fn empty_hand(g: &mut Game, p: Player) {
    let ids: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.owner == p && o.zone == Zone::Hand)
        .map(|(i, _)| i as CardId)
        .collect();
    for id in ids {
        g.move_object(id, Zone::Graveyard);
    }
}
fn end(g: &mut Game) {
    g.state.step = Step::Main2;
    g.give_priority();
    checked(g, Action::Pass);
    checked(g, Action::Pass);
}
#[test]
fn fdn_punisher_two_both_printed_costs_paid_and_quandary_own_casts_do_not_trigger() {
    for p in [Player::P0, Player::P1] {
        for (name, cost) in [("Painful Quandary", 5), ("Perforating Artist", 3)] {
            let mut g = setup(p);
            assert!(Card::from_name(name).unwrap().is_supported());
            let id = named(&mut g, p, name, Zone::Hand);
            let before = g.state.players[p.index()].mana.iter().sum::<u32>();
            cast(&mut g, id, None, &[]);
            settle(&mut g);
            assert_eq!(
                before - g.state.players[p.index()].mana.iter().sum::<u32>(),
                cost
            );
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            if name == "Perforating Artist" {
                assert!(g.keywords(id).contains(Keywords::DEATHTOUCH));
                assert_eq!(
                    g.stats(id),
                    Stats {
                        power: 3,
                        toughness: 2
                    }
                );
            }
            let own = named(&mut g, p, "Bear Cub", Zone::Hand);
            cast(&mut g, own, None, &[]);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 20);
        }
    }
}
#[test]
fn fdn_punisher_two_quandary_discard_decline_empty_and_departed_source_choices_are_nontargeted() {
    for p in [Player::P0, Player::P1] {
        for route in 0..3 {
            let mut g = setup(p.other());
            let q = named(&mut g, p, "Painful Quandary", Zone::Battlefield);
            // A player with hexproof still owes this non-targeted trigger.
            named(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
            let spell = named(
                &mut g,
                p.other(),
                if route == 1 {
                    "Quick Study"
                } else {
                    "Bear Cub"
                },
                Zone::Hand,
            );
            let fodder = named(&mut g, p.other(), "Forest", Zone::Hand);
            cast(&mut g, spell, None, &[]);
            if route == 2 {
                empty_hand(&mut g, p.other());
            }
            // Captured controller/caster must survive a later control change and departure.
            g.state.objects[q as usize].controller = p.other();
            g.move_object(q, Zone::Graveyard);
            let life = g.state.players[p.other().index()].life;
            if route != 2 {
                choice(&mut g);
                assert_eq!(g.actor(), Some(p.other()));
                assert!(!g.observe(p).objects.iter().any(|o| o.id == fodder));
                let illegal = named(&mut g, p, "Forest", Zone::Hand);
                reject(&mut g, Action::Discard { card: illegal });
                if route == 0 {
                    checked(&mut g, Action::Discard { card: fodder });
                } else {
                    checked(&mut g, Action::DeclineEffectPayment);
                }
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                life - if route == 0 { 0 } else { 5 }
            );
            if route == 0 {
                assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
            }
            assert_eq!(g.state.players[p.index()].life, 20);
        }
    }
}
#[test]
fn fdn_punisher_two_artist_raid_off_and_opponent_end_do_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Perforating Artist", Zone::Battlefield);
        end(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        let mut g = setup(p.other());
        named(&mut g, p, "Perforating Artist", Zone::Battlefield);
        g.state.players[p.index()].attacked_this_turn = true;
        end(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn fdn_punisher_two_artist_all_three_routes_empty_options_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        for route in 0..4 {
            let mut g = setup(p);
            let artist = named(&mut g, p, "Perforating Artist", Zone::Battlefield);
            let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
            let permanent = named(&mut g, p.other(), "Swiftfoot Boots", Zone::Battlefield);
            let fodder = named(&mut g, p.other(), "Forest", Zone::Hand);
            if route == 3 {
                empty_hand(&mut g, p.other());
                g.move_object(permanent, Zone::Graveyard);
            }
            g.state.players[p.index()].attacked_this_turn = true;
            end(&mut g);
            if route != 3 {
                choice(&mut g);
                assert_eq!(g.actor(), Some(p.other()));
                g.move_object(artist, Zone::Graveyard);
                let land_ref = g.object_ref(land);
                reject(&mut g, Action::Sacrifice { object: land_ref });
                if route == 0 {
                    let object = g.object_ref(permanent);
                    checked(&mut g, Action::Sacrifice { object });
                } else {
                    checked(&mut g, Action::DeclineEffectPayment);
                    assert_eq!(g.actor(), Some(p.other()));
                    assert!(!g.observe(p).objects.iter().any(|o| o.id == fodder));
                    if route == 1 {
                        checked(&mut g, Action::Discard { card: fodder });
                    } else {
                        checked(&mut g, Action::DeclineEffectPayment);
                    }
                }
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if route < 2 { 20 } else { 17 }
            );
            assert_eq!(g.object_view(land).zone, Zone::Battlefield);
            if route == 0 {
                assert_eq!(g.object_view(permanent).zone, Zone::Graveyard);
            }
            if route == 1 {
                assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn fdn_punisher_two_artist_actual_attack_records_raid_and_deathtouch_kills_larger_blocker() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let artist = named(&mut g, p, "Perforating Artist", Zone::Hand);
        cast(&mut g, artist, None, &[]);
        settle(&mut g);
        let blocker = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        g.state.objects[artist as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: artist });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker,
                attacker: artist,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.object_view(blocker).zone == Zone::Graveyard);
        assert_eq!(g.object_view(artist).zone, Zone::Graveyard);
        assert!(g.state.players[p.index()].attacked_this_turn);
    }
}
