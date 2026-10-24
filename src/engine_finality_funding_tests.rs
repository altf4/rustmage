//! Finality uses the virtual, post-counter-payment exact-object snapshot.
use super::*;
use crate::engine::assurance::big_helpers::{cast, checked, named, settle, setup};
fn goal(p: Player) -> FundingGoal {
    FundingGoal {
        actor: p,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}
#[test]
fn finality_funding_cost_departure_overlay_counts_choose_shuffle_and_do_not_mutate_parent() {
    for p in [Player::P0, Player::P1] {
        for name in ["Bear Cub", "Forest", "Progenitus"] {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Battlefield);
            g.set_counters(id, Counters::of(CounterKind::Finality, 1));
            let before = g.snapshot();
            let mut w = FundingState::from_game(&g, &goal(p));
            w.prepare_entry_projection();
            let reference = w.reference(id);
            let mut destinations = Vec::new();
            w.cost_departures(&[reference], Zone::Graveyard, &mut |after| {
                destinations.push(after.object(id).zone);
                assert_ne!(after.reference(id), reference);
                assert_eq!(after.counter_count(id, CounterKind::Finality), 0);
                assert_eq!(after.creatures_died, 0);
                false
            });
            destinations.sort_by_key(|z| *z as u8);
            assert!(destinations.contains(&Zone::Exile));
            assert_eq!(destinations.contains(&Zone::Library), name == "Progenitus");
            let mut removed = w.clone();
            live_counters::set(&mut removed.named_counters, reference, None);
            let mut destination = None;
            removed.cost_departures(&[reference], Zone::Graveyard, &mut |after| {
                destination = Some(after.object(id).zone);
                false
            });
            assert_eq!(
                destination,
                Some(if name == "Progenitus" {
                    Zone::Library
                } else {
                    Zone::Graveyard
                })
            );
            assert_eq!(g.snapshot(), before);
        }
    }
}
#[test]
fn finality_funding_mana_sacrifice_retains_post_payment_counts_and_real_paid_mana_agrees() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let pizza = named(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
        g.set_counters(pizza, Counters::of(CounterKind::Finality, 2));
        let reference = g.object_ref(pizza);
        let snapshot = g.snapshot();
        for paid_counters in [0, 1, 2] {
            let w = FundingState::from_game(&g, &goal(p));
            let index = w.resources.iter().position(|r| r.id == pizza).unwrap();
            // A typed funding step exercises the common pay-counters + sacrifice
            // boundary. Printed Pizza pays no counters; its real step is row0.
            let step = super::super::FundingMove {
                resource: index,
                ability: Some(0),
                grantor: None,
                borrowed: None,
                payment: [0, 0, 0, 0, 0, 1],
                counters: Counters::of(CounterKind::Finality, paid_counters),
                production: [0, 0, 0, 1, 0, 0],
                quality: None,
                allocation: vec![],
                external: None,
                frozen: None,
            };
            let mut visits = 0;
            w.visit_applied(&step, &mut |after| {
                visits += 1;
                assert_eq!(
                    after.object(pizza).zone,
                    if paid_counters == 2 {
                        Zone::Graveyard
                    } else {
                        Zone::Exile
                    }
                );
                assert_ne!(after.reference(pizza), reference);
                assert_eq!(after.counter_count(pizza, CounterKind::Finality), 0);
                assert_eq!(after.creatures_died, 0);
                false
            });
            assert_eq!(visits, 1);
            assert_eq!(g.snapshot(), snapshot);
        }
        let action = Action::ActivateMana {
            card: pizza,
            ability: 0,
            payment: [0, 0, 0, 0, 0, 1],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 1, 0, 0],
        };
        checked(&mut g, action);
        assert_eq!(g.object_view(pizza).zone, Zone::Exile);
        assert_eq!(g.counter_count(pizza, CounterKind::Finality), 0);
    }
}
#[test]
fn finality_funding_granted_sacrifice_fast_path_excludes_deaths_and_matches_actual() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ring = named(&mut g, p, "Lotus Ring", Zone::Hand);
        cast(&mut g, ring, None, &[]);
        settle(&mut g);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let equip=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..} if *card==ring&&*t==bear)).unwrap();
        checked(&mut g, equip);
        settle(&mut g);
        g.state.objects[bear as usize].summoning_sick = false;
        g.set_counters(bear, Counters::of(CounterKind::Finality, 1));
        let reference = g.object_ref(bear);
        let goal = goal(p);
        let w = FundingState::from_game(&g, &goal);
        let step = super::super::FundingMove {
            resource: w.resources.iter().position(|r| r.id == bear).unwrap(),
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
        let mut visits = 0;
        w.visit_applied(&step, &mut |after| {
            visits += 1;
            assert_eq!(after.object(bear).zone, Zone::Exile);
            assert_ne!(after.reference(bear), reference);
            assert_eq!(after.creatures_died, 0);
            false
        });
        assert_eq!(visits, 1);
        let grantor = g.object_ref(ring);
        checked(
            &mut g,
            Action::TapGrantedMana {
                card: bear,
                grantor,
                production: [0, 0, 0, 3, 0, 0],
            },
        );
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        assert_eq!(g.state.creatures_died_this_turn, 0);
    }
}
#[test]
fn finality_funding_owner_return_entry_counter_replacement_and_hero_layer_persist() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let source = named(
            &mut g,
            p.other(),
            "Thunderbolts Conspiracy",
            Zone::Battlefield,
        );
        let villain = named(&mut g, p, "Ultron Drone", Zone::Graveyard);
        let mut movement = g.entry_move(villain, Zone::Battlefield, false);
        movement.controller = p;
        movement.effect_counter = Some((CounterKind::Finality, 1));
        movement.return_modification =
            Some((crate::cards::ReturnKind::HeroFinality, g.object_ref(source)));
        assert!(Game::entry_choice_subtypes(&movement).has(movement.before.card, "Hero"));
        let snapshot = g.snapshot();
        let w = FundingState::from_game(&g, &goal(p));
        let mut visits = 0;
        w.visit_entry_moves(vec![movement], true, None, &mut |after| {
            visits += 1;
            assert_eq!(after.object(villain).zone, Zone::Battlefield);
            assert_eq!(after.object(villain).controller, p);
            assert_eq!(after.counter_count(villain, CounterKind::Finality), 2);
            assert!(
                after
                    .subtypes(villain)
                    .has(after.object(villain).card, "Hero")
            );
            let mut without_source = after.clone();
            let i = without_source
                .resources
                .iter()
                .position(|r| r.id == source)
                .unwrap();
            without_source.resources[i].object.zone = Zone::Exile;
            assert!(
                without_source
                    .subtypes(villain)
                    .has(without_source.object(villain).card, "Hero")
            );
            without_source.resources[i].object.controller = p;
            assert!(
                without_source
                    .subtypes(villain)
                    .has(without_source.object(villain).card, "Hero")
            );
            false
        });
        assert_eq!(visits, 1);
        assert_eq!(g.snapshot(), snapshot);
    }
}
