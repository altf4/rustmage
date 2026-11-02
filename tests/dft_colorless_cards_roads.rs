mod support;
use rustmage::Card;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ROADS: [(&str, usize); 5] = [
    ("Country Roads", 0),
    ("Reef Roads", 1),
    ("Foul Roads", 2),
    ("Rocky Roads", 3),
    ("Wild Roads", 4),
];

fn sacrifice(land: CardId) -> Action {
    Action::AnnounceActivate {
        card: land,
        ability: 1,
        counter_payment: Counters::default(),
    }
}

#[test]
fn roads_entry_checks_the_actual_controller_and_either_subtype() {
    for (name, _) in ROADS {
        for p in [Player::P0, Player::P1] {
            for permanent in [None, Some("Trained Arynx"), Some("Skybox Ferry")] {
                for own in [false, true] {
                    let mut g = scenario(p);
                    if let Some(other) = permanent {
                        put(
                            &mut g,
                            if own { p } else { p.other() },
                            other,
                            Zone::Battlefield,
                        );
                    }
                    let land = put(&mut g, p, name, Zone::Hand);
                    act(&mut g, Action::PlayLand { card: land });
                    assert_eq!(g.object_view(land).zone, Zone::Battlefield);
                    assert_eq!(
                        g.object_view(land).tapped,
                        permanent.is_none() || !own,
                        "{name}"
                    );
                }
            }
        }
    }
}

#[test]
fn roads_pay_colored_sorcery_cost_and_create_the_real_pilot() {
    for (name, color) in ROADS {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p);
            let land = put(&mut g, p, name, Zone::Battlefield);
            g.state.players[p.index()].mana = [0; 6];
            let mana = g
                .legal_actions()
                .into_iter()
                .find(|action| {
                    matches!(action,
                Action::ActivateMana { card, ability: 0, .. } if *card == land)
                })
                .unwrap();
            let before = g.snapshot();
            act(&mut g, mana);
            let mut expected = [0; 6];
            expected[color] = 1;
            assert_eq!(g.state.players[p.index()].mana, expected, "{name}");
            assert!(g.object_view(land).tapped);
            g.restore(&before).unwrap();
            g.state.players[p.index()].mana[5] = 2;
            reject(&mut g, sacrifice(land));
            g.state.players[p.index()].mana = expected;
            g.state.players[p.index()].mana[5] = 1;
            let mut wrong_time = g.clone();
            wrong_time.state.step = Step::Upkeep;
            reject(&mut wrong_time, sacrifice(land));
            let mut tapped = g.clone();
            tapped.state.objects[land as usize].tapped = true;
            reject(&mut tapped, sacrifice(land));
            act(&mut g, sacrifice(land));
            pay(&mut g);
            assert_eq!(
                g.object_view(land).zone,
                Zone::Graveyard,
                "cost is paid before resolution"
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            let pilots = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, object)| {
                    object.zone == Zone::Battlefield
                        && object.controller == p
                        && object.card == Card::from_name("@token/pilot_1_1_colorless").unwrap()
                })
                .map(|(id, _)| id as CardId)
                .collect::<Vec<_>>();
            assert_eq!(pilots.len(), 1, "{name}");
            let pilot = pilots[0];
            let view = g.object_view(pilot);
            assert!(view.token.is_some());
            assert!(!view.tapped);
            assert_eq!(stats(&g, pilot), (1, 1));
            assert_eq!(view.colors, 0);
            assert!(g.has_subtype(pilot, "Pilot"));
            assert!(g.keywords(pilot).contains(Keywords::CREW_SADDLE_PLUS_TWO));
        }
    }
}

#[test]
fn country_roads_pilot_actually_crews_and_saddles_on_owned_branches() {
    for p in [Player::P0, Player::P1] {
        let mut parent = scenario(p);
        let land = put(&mut parent, p, "Country Roads", Zone::Battlefield);
        parent.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        act(&mut parent, sacrifice(land));
        let saved = parent.snapshot();
        let payment = parent
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let mut restored = parent.clone();
        restored.restore(&saved).unwrap();
        drop(parent);
        for (i, branch) in [&mut raw, &mut indexed, &mut restored]
            .into_iter()
            .enumerate()
        {
            if i == 1 {
                let at = branch
                    .legal_actions()
                    .iter()
                    .position(|a| *a == payment)
                    .unwrap();
                branch.apply_index(p, branch.revision(), at).unwrap();
            } else {
                act(branch, payment.clone());
            }
            settle(branch);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.snapshot(), restored.snapshot());
        let pilot = raw
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, object)| {
                object.zone == Zone::Battlefield && object.controller == p && object.card.is_token()
            })
            .unwrap()
            .0 as CardId;
        for crew in [false, true] {
            let mut g = raw.clone();
            let source = put(
                &mut g,
                p,
                if crew {
                    "Clamorous Ironclad"
                } else {
                    "Dracosaur Auxiliary"
                },
                Zone::Battlefield,
            );
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            if matches!(g.state.pending, Pending::PayCosts { .. }) {
                pay(&mut g);
            }
            let object = g.object_ref(pilot);
            act(
                &mut g,
                if crew {
                    Action::ChooseCrewer { object }
                } else {
                    Action::ChooseSaddler { object }
                },
            );
            act(
                &mut g,
                if crew {
                    Action::FinishCrew
                } else {
                    Action::FinishSaddle
                },
            );
            settle(&mut g);
            assert!(
                g.object_view(pilot).tapped,
                "one power plus two pays the full three"
            );
            if crew {
                assert!(g.is_creature(source));
            } else {
                assert!(g.object_view(source).saddled);
            }
        }
    }
}
