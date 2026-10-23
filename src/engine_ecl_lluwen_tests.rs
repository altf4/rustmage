//! Exact mill recovery and paid hybrid activation, privacy and owned branches.
use super::ability_timeline_tests::{checked, named, settle, setup};
use super::ability_world::CharacteristicWorld;
use super::*;
const NAME: &str = "Lluwen, Imperfect Naturalist";
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn cast_lluwen(g: &mut Game, id: CardId) {
    let p = g.state.objects[id as usize].owner;
    checked(g, Action::AnnounceCast { card: id });
    checked(
        g,
        Action::ChooseSpellCost {
            cost: g.state.players[p.index()].mana[2].min(2) as u16,
        },
    );
    pay(g);
}
fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, action);
}
fn activate(g: &mut Game, id: CardId, land: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let p = g.state.objects[id as usize].controller;
    checked(
        g,
        Action::ChooseActivationCost {
            cost: g.state.players[p.index()].mana[2].min(3) as u16,
        },
    );
    for _ in 0..4 {
        let action = Action::ChooseAdditionalCost {
            object: g.object_ref(land),
        };
        if g.legal_actions().contains(&action) {
            checked(g, action);
            break;
        }
        checked(g, Action::FinishTargets);
    }
    pay(g);
}
fn paid(g: &mut Game, p: Player, mana: Mana) -> CardId {
    let id = named(g, p, NAME, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast_lluwen(g, id);
    settle(g);
    id
}
#[test]
fn ecl_lluwen_all_paid_hybrid_cast_and_activation_routes_include_discarded_land() {
    for p in [Player::P0, Player::P1] {
        for black in 0..=2 {
            for activation_black in 0..=3 {
                let mut g = setup(p);
                g.state.players[p.index()].library.clear();
                let id = paid(&mut g, p, [0, 0, black, 0, 2 - black, 0]);
                assert_eq!(
                    g.stats(id),
                    Stats {
                        power: 1,
                        toughness: 3
                    }
                );
                g.state.objects[id as usize].summoning_sick = false;
                let land = named(&mut g, p, "Forest", Zone::Hand);
                named(&mut g, p, "Swamp", Zone::Graveyard);
                named(&mut g, p.other(), "Forest", Zone::Graveyard);
                g.state.players[p.index()].mana =
                    [0, 0, activation_black, 0, 3 - activation_black, 2];
                activate(&mut g, id, land);
                assert!(g.object_view(id).tapped);
                assert_eq!(g.state.objects[land as usize].zone, Zone::Graveyard);
                settle(&mut g);
                let worms: Vec<_> = g
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(_, o)| {
                        o.zone == Zone::Battlefield
                            && o.controller == p
                            && o.is_token()
                            && o.card
                                == Card::from_name("@token/lluwen_worm_1_1_black_green").unwrap()
                    })
                    .map(|(i, _)| i as CardId)
                    .collect();
                assert_eq!(worms.len(), 2);
                for worm in worms {
                    assert_eq!(
                        g.stats(worm),
                        Stats {
                            power: 1,
                            toughness: 1
                        }
                    );
                    assert_eq!(g.colors(worm), 20);
                    assert!(g.has_subtype(worm, "Worm"));
                    assert!(!g.object_view(worm).tapped);
                }
            }
        }
    }
}
#[test]
fn ecl_lluwen_optional_mill_recovery_is_one_exact_public_creature_or_land() {
    for p in [Player::P0, Player::P1] {
        for take in [None, Some(0usize), Some(1usize)] {
            let mut g = setup(p);
            g.state.players[p.index()].library.clear();
            let old = named(&mut g, p, "Forest", Zone::Graveyard);
            let bottom = named(&mut g, p, "Shock", Zone::Library);
            let land = named(&mut g, p, "Forest", Zone::Library);
            let creature = named(&mut g, p, "Bear Cub", Zone::Library);
            let instant = named(&mut g, p, "Shock", Zone::Library);
            let enchantment = named(&mut g, p, "Meek Attack", Zone::Library);
            g.state.players[p.index()].library =
                vec![bottom, land, creature, instant, enchantment].into();
            let id = named(&mut g, p, NAME, Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 1, 0];
            cast_lluwen(&mut g, id);
            for _ in 0..12 {
                if matches!(g.state.pending, Pending::MilledSelection { .. }) {
                    break;
                }
                let action = if g.state.pending == Pending::Priority {
                    Action::Pass
                } else {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PutTrigger { .. }))
                        .unwrap()
                };
                checked(&mut g, action);
            }
            let choice = g.observe(p).milled_choice.unwrap();
            assert_eq!(choice.candidates.len(), 2);
            assert!(g.observe(p.other()).milled_choice.is_none());
            for invalid in [old, instant, enchantment] {
                let object = g.object_ref(invalid);
                reject(&mut g, Action::SelectMilled { object });
            }
            let stale = ObjectRef {
                card: land,
                generation: g.object_ref(land).generation - 1,
            };
            reject(&mut g, Action::SelectMilled { object: stale });
            // The trigger remains independent after the source leaves.
            g.move_object(id, Zone::Graveyard);
            if let Some(index) = take {
                let chosen = [land, creature][index];
                let object = g.object_ref(chosen);
                checked(&mut g, Action::SelectMilled { object });
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].library.last(), Some(&chosen));
                for viewer in [p, p.other()] {
                    assert_eq!(
                        g.observe(viewer).known_library_top[p.index()]
                            .as_ref()
                            .unwrap()
                            .object,
                        g.object_ref(chosen)
                    );
                }
            } else {
                checked(&mut g, Action::FinishMilledSelection);
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].library.as_slice(), &[bottom]);
            }
        }
    }
}
#[test]
fn ecl_lluwen_short_empty_and_nonmatching_mill_have_no_mandatory_choice() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=3 {
            let mut g = setup(p);
            g.state.players[p.index()].library.clear();
            let ids: Vec<_> = (0..count)
                .map(|_| named(&mut g, p, "Shock", Zone::Library))
                .collect();
            let id = paid(&mut g, p, [0, 0, 2, 0, 0, 0]);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
            assert!(g.state.players[p.index()].library.is_empty());
            for card in ids {
                assert_eq!(g.state.objects[card as usize].zone, Zone::Graveyard);
            }
            assert!(g.observe(p).milled_choice.is_none());
        }
    }
}
#[test]
fn ecl_lluwen_land_discard_cost_rejects_wrong_cards_and_tap_constraints_atomically() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].library.clear();
        let id = paid(&mut g, p, [0, 0, 0, 0, 2, 0]);
        g.state.players[p.index()].mana = [30; 6];
        let announce = Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        };
        let land = named(&mut g, p, "Forest", Zone::Hand);
        assert!(
            !g.legal_actions().contains(&announce),
            "tap symbol obeys sickness"
        );
        g.state.objects[id as usize].summoning_sick = false;
        let bad = named(&mut g, p, "Bear Cub", Zone::Hand);
        let enemy = named(&mut g, p.other(), "Forest", Zone::Hand);
        let grave = named(&mut g, p, "Forest", Zone::Graveyard);
        checked(&mut g, announce);
        checked(&mut g, Action::ChooseActivationCost { cost: 0 });
        while g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayCosts { .. }))
        );
        for card in [bad, enemy, grave] {
            let object = g.object_ref(card);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        let object = g.object_ref(land);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        settle(&mut g);
        assert!(g.object_view(id).tapped);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==id))
        );
    }
}
#[test]
fn ecl_lluwen_resolution_counts_live_graveyard_and_retains_captured_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].library.clear();
        let id = paid(&mut g, p, [0, 0, 2, 0, 0, 0]);
        g.state.objects[id as usize].summoning_sick = false;
        let land = named(&mut g, p, "Forest", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 3, 0, 0, 2];
        activate(&mut g, id, land);
        g.move_object(land, Zone::Exile);
        let extra = named(&mut g, p, "Swamp", Zone::Graveyard);
        let item = g.activation_item(p, id, 0);
        g.gain_control(&item, Subject::Source, PlayerSet::Opponent);
        g.move_object(id, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.objects[extra as usize].zone, Zone::Graveyard);
        let worms: Vec<_> = g
            .state
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.card == Card::from_name("@token/lluwen_worm_1_1_black_green").unwrap()
            })
            .collect();
        assert_eq!(worms.len(), 1);
        assert_eq!(worms[0].controller, p);
    }
}
#[test]
fn ecl_lluwen_opponent_turn_activation_can_resolve_with_zero_lands() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].library.clear();
        let id = paid(&mut g, p, [0, 0, 2, 0, 0, 0]);
        g.state.objects[id as usize].summoning_sick = false;
        let land = named(&mut g, p, "Forest", Zone::Hand);
        g.state.active = p.other();
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 3, 0, 0, 2];
        activate(&mut g, id, land);
        g.move_object(land, Zone::Exile);
        settle(&mut g);
        assert!(!g.state.players[p.index()].token_creation_attempted_this_turn);
        assert!(
            !g.state
                .objects
                .iter()
                .any(|o| o.is_token() && o.zone == Zone::Battlefield)
        );
    }
}
