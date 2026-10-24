//! Virtual additional-cost departures agree with real modal-face movement.
use super::super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;

#[test]
fn msh_faces_cost_departures_restore_front_after_freezing_back_lki() {
    for p in [Player::P0, Player::P1] {
        for (requested, replacement, destination) in [
            (Zone::Graveyard, false, Zone::Graveyard),
            (Zone::Graveyard, true, Zone::Exile),
            (Zone::Exile, false, Zone::Exile),
        ] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Monica Rambeau", Zone::Hand);
            checked(&mut g, Action::AnnounceBack { card: id });
            while g.state.announcing.is_some() {
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                    .unwrap();
                checked(&mut g, action);
            }
            settle(&mut g);
            if replacement {
                let rip = named(&mut g, p, "Rest in Peace", Zone::Hand);
                cast(&mut g, rip, None, &[]);
                settle(&mut g);
            }
            g.set_counters(id, Counters::of(CounterKind::PlusOne, 2));
            let reference = g.object_ref(id);
            let before = g.snapshot();
            let back = mana_lots::Source::in_world(&g, id);
            assert_eq!(back.card, Card::MshPhotonLivingLight);
            assert_eq!(back.card.fixed_mana_value(), 5);
            assert_eq!(back.card.colors(), 9);
            assert!(back.subtypes.has(back.card, "Elemental"));
            assert!(!back.subtypes.has(back.card, "Human"));

            // The actual object is owned independently of the funding parent.
            let mut actual = {
                let parent = g.clone();
                parent.clone()
            };
            let event = actual.move_object_raw(id, requested);
            assert!(event.moved);
            assert_eq!(event.destination, destination);
            assert_eq!(event.before.reference, reference);
            assert_eq!(event.before.card, back.card);
            assert_eq!(event.before.name(), "Photon, Living Light");
            assert_eq!(event.before.counters.get(CounterKind::PlusOne), 2);
            let expected = actual.object_view(id);
            assert_eq!(expected.name(), "Monica Rambeau");
            assert_eq!(expected.card.fixed_mana_value(), 3);
            assert_eq!(expected.colors, 1);
            assert!(expected.granted_subtypes.has(expected.card, "Human"));
            assert!(!expected.granted_subtypes.has(expected.card, "Elemental"));

            let goal = FundingGoal {
                actor: p,
                required: [0; 6],
                generic: 0,
                reserve: None,
                source: None,
            };
            let mut world = FundingState::from_game(&g, &goal);
            world.prepare_entry_projection();
            // Include this exact surrounding payment source in the normal LKI
            // capture path without inventing a new activated ability on Photon.
            world.payment_source = Some(reference);
            let mut visits = 0;
            world.cost_departures(&[reference], requested, &mut |departed| {
                visits += 1;
                let view =
                    characteristic_query::CharacteristicQuery::in_world(departed).object_view(id);
                assert_eq!(view.reference, expected.reference);
                assert_eq!(view.zone, expected.zone);
                assert_eq!(view.card, expected.card);
                assert_eq!(view.name(), expected.name());
                assert_eq!(view.colors, expected.colors);
                assert_eq!(view.kinds, expected.kinds);
                assert_eq!(view.granted_subtypes, expected.granted_subtypes);
                assert_eq!(view.counters, expected.counters);
                assert_eq!(view.controller, expected.controller);
                assert_eq!(
                    (view.power, view.toughness),
                    (expected.power, expected.toughness)
                );
                assert_eq!(view.face, expected.face);
                let saved = departed
                    .payment_departures
                    .iter()
                    .find(|(object, _)| *object == reference)
                    .unwrap()
                    .1;
                assert_eq!(saved, back);
                assert_ne!(saved.card, view.card);
                false
            });
            assert_eq!(visits, 1);
            assert_eq!(g.snapshot(), before);
            let after = actual.snapshot();
            actual.restore(&before).unwrap();
            assert_eq!(actual.object_view(id).card, back.card);
            actual.move_object_raw(id, requested);
            assert_eq!(actual.snapshot(), after);
        }
    }
}
