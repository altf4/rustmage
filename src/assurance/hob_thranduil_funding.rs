//! New virtual graveyard donors must be visible before actual mana payment.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
#[test]
fn thranduil_funding_virtual_elf_sacrifice_unlocks_mana_without_live_game_fallback() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = named(&mut g, p, "Thranduil, the Elvenking", Zone::Battlefield);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let ring = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
        let grantor = g.object_ref(ring);
        assert!(g.attach_equipment(grantor, g.object_ref(elf)));
        g.state.players[p.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 4, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        assert!(g.can_fund(goal.clone()));
        let saved = g.snapshot();
        let world = FundingState::from_game(&g, &goal);
        let step = world
            .moves(&goal)
            .into_iter()
            .find(|s| {
                world.resources[s.resource].id == elf
                    && s.grantor == Some(grantor)
                    && s.production == [0, 0, 0, 0, 3, 0]
            })
            .expect("sacrifice mana source");
        let mut observed = false;
        let mut expected = None;
        world.visit_applied(&step, &mut |next| {
            assert_eq!(next.object(elf).zone, Zone::Graveyard);
            assert_eq!(g.object_view(elf).zone, Zone::Battlefield);
            let ability = borrowed_activations::abilities(next, t)
                .into_iter()
                .find(|a| a.donor.card == elf && a.ability == u16::MAX)
                .expect("new virtual donor");
            assert_eq!(ability.zone, Zone::Graveyard);
            assert_eq!(ability.donor, next.reference(elf));
            let borrow_move = next
                .moves(&goal)
                .into_iter()
                .find(|m| {
                    m.borrowed
                        .is_some_and(|b| b.donor == ability.donor && b.slot == ability.slot)
                })
                .expect("borrowed mana payment");
            next.visit_applied(&borrow_move, &mut |paid| {
                assert!(paid.borrowed_uses.contains(&(ability.source, ability.slot)));
                expected = Some((ability.donor, ability.slot));
                true
            });
            assert!(
                next.moves(&goal)
                    .iter()
                    .any(|s| next.resources[s.resource].id == t
                        && s.borrowed.is_some_and(|b| b.donor == ability.donor)
                        && s.production == [0, 0, 0, 0, 1, 0])
            );
            observed = true;
            true
        });
        assert!(observed);
        assert_eq!(g.snapshot(), saved);
        drop(world);
        g.apply(
            p,
            Action::TapGrantedMana {
                card: elf,
                grantor,
                production: [0, 0, 0, 0, 3, 0],
            },
        )
        .unwrap();
        assert_eq!(g.object_view(elf).zone, Zone::Graveyard);
        let (predicted_donor, predicted_slot) = expected.unwrap();
        let graveyard = g.object_ref(elf);
        assert_eq!(graveyard, predicted_donor);
        let acquired = g
            .graveyard_borrowed_ability_views()
            .unwrap()
            .into_iter()
            .find(|a| a.source.card == t && a.graveyard == graveyard && a.ability == u16::MAX)
            .unwrap();
        assert_eq!(acquired.slot, predicted_slot);
        assert!(!acquired.used);
        g.apply(
            p,
            Action::AnnounceGraveyardBorrowedMana {
                card: t,
                graveyard,
                ability: u16::MAX,
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            },
        )
        .unwrap();
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .expect("zero payment");
            g.apply(g.actor().unwrap(), a).unwrap();
        }
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 4, 0]);
        assert!(g.object_view(t).tapped);
        let used = g
            .graveyard_borrowed_ability_views()
            .unwrap()
            .into_iter()
            .find(|a| a.source.card == t && a.graveyard == graveyard && a.ability == u16::MAX)
            .unwrap();
        assert_eq!(used.slot, predicted_slot);
        assert!(used.used);
        let completed = g.snapshot();
        g.restore(&saved).unwrap();
        g.apply(
            p,
            Action::TapGrantedMana {
                card: elf,
                grantor,
                production: [0, 0, 0, 0, 3, 0],
            },
        )
        .unwrap();
        g.apply(
            p,
            Action::AnnounceGraveyardBorrowedMana {
                card: t,
                graveyard: g.object_ref(elf),
                ability: u16::MAX,
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            },
        )
        .unwrap();
        while g.state.announcing.is_some() {
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            g.apply(g.actor().unwrap(), action).unwrap();
        }
        assert_eq!(g.snapshot(), completed);
    }
}

#[test]
fn sacrificed_stolen_mana_source_projection_matches_owned_zone_identity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elf = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        g.state.objects[elf as usize].controller = p;
        g.state.objects[elf as usize].summoning_sick = false;
        g.state.objects[elf as usize].growth = Stats {
            power: 2,
            toughness: 2,
        };
        g.state.objects[elf as usize].damage = 1;
        g.state.objects[elf as usize].damaged_this_turn = true;
        g.state.objects[elf as usize].gained_keywords = Keywords::HASTE;
        let ring = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
        let grantor = g.object_ref(ring);
        assert!(g.attach_equipment(grantor, g.object_ref(elf)));
        g.state.players[p.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 3, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let saved = g.snapshot();
        let world = FundingState::from_game(&g, &goal);
        let step = world
            .moves(&goal)
            .into_iter()
            .find(|step| {
                world.resources[step.resource].id == elf
                    && step.grantor == Some(grantor)
                    && step.production == [0, 0, 0, 0, 3, 0]
            })
            .expect("sacrifice mana source");
        let mut predicted = None;
        world.visit_applied(&step, &mut |next| {
            predicted = Some(next.object(elf).clone());
            true
        });
        drop(world);
        g.apply(
            p,
            Action::TapGrantedMana {
                card: elf,
                grantor,
                production: [0, 0, 0, 0, 3, 0],
            },
        )
        .unwrap();
        assert_eq!(predicted.unwrap(), g.state.objects[elf as usize]);
        assert_eq!(g.state.objects[elf as usize].controller, p.other());
        g.restore(&saved).unwrap();
        assert_eq!(g.snapshot(), saved);
    }
}

#[test]
fn sacrificed_manifest_reveals_physical_elf_for_projected_graveyard_borrowing() {
    for p in [Player::P0, Player::P1] {
        for copied in [false, true] {
            let mut g = setup(p);
            let thranduil = named(&mut g, p, "Thranduil, the Elvenking", Zone::Battlefield);
            let elf = named(&mut g, p, "Llanowar Elves", Zone::Library);
            let movement = g.manifest_entry(g.object_ref(elf), p).unwrap();
            g.stage_entries(vec![movement], None);
            assert!(!g.advance_entries());
            assert!(g.is_face_down(g.object_ref(elf)));
            if copied {
                g.apply_temporary_copy(
                    g.object_ref(elf),
                    CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
                    CopyDeadline::EndOfTurn,
                );
            }
            g.state.objects[elf as usize].summoning_sick = false;
            let ring = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
            let grantor = g.object_ref(ring);
            assert!(g.attach_equipment(grantor, g.object_ref(elf)));
            g.state.players[p.index()].mana = [0; 6];
            let goal = FundingGoal {
                actor: p,
                required: [0, 0, 0, 0, 4, 0],
                generic: 0,
                reserve: None,
                source: None,
            };
            assert!(g.can_fund(goal.clone()));
            let before = g.snapshot();
            let world = FundingState::from_game(&g, &goal);
            let step = world
                .moves(&goal)
                .into_iter()
                .find(|step| {
                    world.resources[step.resource].id == elf
                        && step.grantor == Some(grantor)
                        && step.production == [0, 0, 0, 0, 3, 0]
                })
                .unwrap();
            let mut predicted = None;
            world.visit_applied(&step, &mut |next| {
                let donor = borrowed_activations::abilities(next, thranduil)
                    .into_iter()
                    .find(|a| a.donor.card == elf && a.ability == u16::MAX)
                    .unwrap();
                predicted = Some((next.object(elf).clone(), donor.slot));
                true
            });
            drop(world);
            g.apply(
                p,
                Action::TapGrantedMana {
                    card: elf,
                    grantor,
                    production: [0, 0, 0, 0, 3, 0],
                },
            )
            .unwrap();
            let (predicted_object, predicted_slot) = predicted.unwrap();
            assert_eq!(predicted_object, g.state.objects[elf as usize]);
            assert_eq!(g.state.objects[elf as usize].card.name(), "Llanowar Elves");
            let actual = g
                .graveyard_borrowed_ability_views()
                .unwrap()
                .into_iter()
                .find(|a| {
                    a.source.card == thranduil && a.graveyard.card == elf && a.ability == u16::MAX
                })
                .unwrap();
            assert_eq!(actual.slot, predicted_slot);
            g.restore(&before).unwrap();
            assert_eq!(g.snapshot(), before);
        }
    }
}

#[test]
fn projected_manifest_entry_then_sacrifice_matches_owned_borrowing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let thranduil = named(&mut g, p, "Thranduil, the Elvenking", Zone::Battlefield);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Library);
        let ring = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
        let grantor = g.object_ref(ring);
        let movement = g.manifest_entry(g.object_ref(elf), p).unwrap();
        g.state.players[p.index()].mana = [0; 6];
        let saved = g.snapshot();
        let goal = FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 4, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&g, &goal);
        let mut predicted = None;
        world.commit_projected_entries(vec![movement.clone()], &[0], &mut |entered| {
            let mut next = entered.clone();
            let source = next.reference(elf);
            let elf_resource = next.resources.iter_mut().find(|r| r.id == elf).unwrap();
            elf_resource.object.summoning_sick = false;
            let ring_resource = next.resources.iter_mut().find(|r| r.id == ring).unwrap();
            ring_resource.object.attached_to = Some(source);
            next.refresh_control_projection();
            let step = next
                .moves(&goal)
                .into_iter()
                .find(|step| {
                    next.resources[step.resource].id == elf
                        && step.grantor == Some(grantor)
                        && step.production == [0, 0, 0, 0, 3, 0]
                })
                .expect("projected sacrifice");
            next.visit_applied(&step, &mut |departed| {
                let donor = borrowed_activations::abilities(departed, thranduil)
                    .into_iter()
                    .find(|a| a.donor.card == elf && a.ability == u16::MAX)
                    .unwrap();
                predicted = Some((departed.object(elf).clone(), donor.slot));
                true
            });
            true
        });
        drop(world);
        g.stage_entries(vec![movement], None);
        assert!(!g.advance_entries());
        g.state.objects[elf as usize].summoning_sick = false;
        assert!(g.attach_equipment(grantor, g.object_ref(elf)));
        g.apply(
            p,
            Action::TapGrantedMana {
                card: elf,
                grantor,
                production: [0, 0, 0, 0, 3, 0],
            },
        )
        .unwrap();
        let (object, slot) = predicted.unwrap();
        assert_eq!(object, g.state.objects[elf as usize]);
        let actual = g
            .graveyard_borrowed_ability_views()
            .unwrap()
            .into_iter()
            .find(|a| {
                a.source.card == thranduil && a.graveyard.card == elf && a.ability == u16::MAX
            })
            .unwrap();
        assert_eq!(actual.slot, slot);
        let completed = g.snapshot();
        g.restore(&saved).unwrap();
        let movement = g.manifest_entry(g.object_ref(elf), p).unwrap();
        g.stage_entries(vec![movement], None);
        assert!(!g.advance_entries());
        g.state.objects[elf as usize].summoning_sick = false;
        assert!(g.attach_equipment(grantor, g.object_ref(elf)));
        g.apply(
            p,
            Action::TapGrantedMana {
                card: elf,
                grantor,
                production: [0, 0, 0, 0, 3, 0],
            },
        )
        .unwrap();
        assert_eq!(g.snapshot(), completed);
    }
}
