//! Face normalization must wait for the post-payment finality replacement snapshot.
use super::*;
use crate::engine::assurance::big_helpers::{cast, checked, named, settle, setup};

#[test]
fn msh_combined_granted_mana_departure_preserves_back_until_finality_planning() {
    for p in [Player::P0, Player::P1] {
        for finality in [false, true] {
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
            assert_eq!(g.object_view(id).card, Card::MshPhotonLivingLight);
            let ring = named(&mut g, p, "Lotus Ring", Zone::Hand);
            cast(&mut g, ring, None, &[]);
            settle(&mut g);
            let equip = g.legal_actions().into_iter().find(|a| matches!(a, Action::Activate { card, target: Some(Target::Card(t)), .. } if *card == ring && *t == id)).unwrap();
            checked(&mut g, equip);
            settle(&mut g);
            g.state.objects[id as usize].summoning_sick = false;
            if finality {
                g.set_counters(id, Counters::of(CounterKind::Finality, 1));
            }
            let before = g.snapshot();
            let goal = FundingGoal {
                actor: p,
                required: [0; 6],
                generic: 0,
                reserve: None,
                source: None,
            };
            let world = FundingState::from_game(&g, &goal);
            let step = super::super::FundingMove {
                resource: world.resources.iter().position(|r| r.id == id).unwrap(),
                ability: None,
                grantor: Some(g.object_ref(ring)),
                borrowed: None,
                payment: [0; 6],
                counters: Counters::default(),
                production: [0, 0, 0, 3, 0, 0],
                quality: None,
                allocation: vec![],
                external: None,
                frozen: None,
            };
            let mut intermediate = world.clone();
            intermediate.apply(&step, true);
            // The slow path restores this temporary zone before replacements.
            // Keep the real back-face characteristics until that planner freezes facts.
            assert_eq!(intermediate.object(id).card, Card::MshPhotonLivingLight);
            assert_eq!(
                intermediate.counter_count(id, CounterKind::Finality),
                u64::from(finality)
            );
            let expected_zone = if finality {
                Zone::Exile
            } else {
                Zone::Graveyard
            };
            let mut visits = 0;
            world.visit_applied(&step, &mut |after| {
                visits += 1;
                assert_eq!(after.object(id).zone, expected_zone);
                assert_eq!(
                    after.object(id).card,
                    Card::from_name("Monica Rambeau").unwrap()
                );
                assert_eq!(after.object(id).card.fixed_mana_value(), 3);
                assert_eq!(after.counter_count(id, CounterKind::Finality), 0);
                assert!(after.subtypes(id).has(after.object(id).card, "Human"));
                assert!(!after.subtypes(id).has(after.object(id).card, "Elemental"));
                false
            });
            assert_eq!(visits, 1);
            assert_eq!(g.snapshot(), before);
            let grantor = g.object_ref(ring);
            checked(
                &mut g,
                Action::TapGrantedMana {
                    card: id,
                    grantor,
                    production: [0, 0, 0, 3, 0, 0],
                },
            );
            assert_eq!(g.object_view(id).zone, expected_zone);
            assert_eq!(g.object_view(id).name(), "Monica Rambeau");
            assert_eq!(g.state.creatures_died_this_turn, u64::from(!finality));
        }
    }
}
