//! Real gated definitions exercise the core; support flags remain false.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::tests::spawn;
use super::*;

fn gated(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(!card.is_supported(), "core fixtures stay gated");
    spawn(g, p, card, zone)
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn attach(g: &mut Game, equipment: CardId, creature: CardId) {
    assert!(g.attach_equipment(g.object_ref(equipment), g.object_ref(creature)));
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn hand_myojin(g: &mut Game, p: Player) -> CardId {
    let id = gated(g, p, "Myojin of Night's Reach", Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn venom(g: &mut Game, p: Player) -> CardId {
    let id = gated(g, p, "Venom Connoisseur", Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn until_stack(g: &mut Game, length: usize) {
    g.prepare_triggers();
    for _ in 0..80 {
        if g.state.stack.len() == length && g.state.pending == Pending::Priority {
            return;
        }
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. } | Action::FinishTargets | Action::PayCosts { .. }
                    )
                })
                .unwrap()
        };
        checked(g, a);
    }
    panic!("did not reach stack length {length}");
}

#[test]
fn fdn_bounded_core_fiery_targets_at_cast_only_related_equipment_and_exiles_lethal_creature() {
    for p in [Player::P0, Player::P1] {
        for optional in [false, true] {
            let mut g = setup(p);
            let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let other = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let bow = named(&mut g, p.other(), "Short Bow", Zone::Battlefield);
            let wrong = named(&mut g, p.other(), "Short Bow", Zone::Battlefield);
            attach(&mut g, bow, bear);
            attach(&mut g, wrong, other);
            let fiery = gated(&mut g, p, "Fiery Annihilation", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: fiery });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(wrong)
            }));
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(wrong),
                },
            );
            if optional {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(bow),
                    },
                );
            }
            while g.state.announcing.is_some() {
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                    .unwrap();
                checked(&mut g, a);
            }
            assert_eq!(
                g.state.stack.last().unwrap().targets.len(),
                if optional { 2 } else { 1 }
            );
            settle(&mut g);
            assert_eq!(g.object_view(bear).zone, Zone::Exile);
            assert_eq!(
                g.object_view(bow).zone,
                if optional {
                    Zone::Exile
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.object_view(wrong).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn fdn_bounded_core_fiery_rechecks_attachment_and_independent_target_legality() {
    for p in [Player::P0, Player::P1] {
        for protect_creature in [false, true] {
            let mut g = setup(p);
            let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let other = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let bow = named(&mut g, p.other(), "Short Bow", Zone::Battlefield);
            attach(&mut g, bow, bear);
            let fiery = gated(&mut g, p, "Fiery Annihilation", Zone::Hand);
            cast(
                &mut g,
                fiery,
                None,
                &[Target::Card(bear), Target::Card(bow)],
            );
            if protect_creature {
                checked(&mut g, Action::Pass);
                let veil = named(&mut g, p.other(), "Snakeskin Veil", Zone::Hand);
                cast(&mut g, veil, None, &[Target::Card(bear)]);
            } else {
                // Moving an Equipment after announcement changes only its relation.
                attach(&mut g, bow, other);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(bear).zone,
                if protect_creature {
                    Zone::Battlefield
                } else {
                    Zone::Exile
                }
            );
            assert_eq!(
                g.object_view(bow).zone,
                if protect_creature {
                    Zone::Exile
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
#[test]
fn fdn_bounded_core_myojin_true_hand_cast_named_replacements_and_source_only_keywords() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = hand_myojin(&mut g, p);
        assert_eq!(g.counter_count(id, CounterKind::Divinity), 1);
        assert!(g.keywords(id).contains(Keywords::INDESTRUCTIBLE));
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(bear, CounterKind::Divinity, 1);
        assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        let other = gated(&mut g, p, "Myojin of Night's Reach", Zone::Graveyard);
        // Direct reanimation is not casting, even following a hand cast this step.
        g.move_object(other, Zone::Battlefield);
        assert_eq!(g.counter_count(other, CounterKind::Divinity), 0);
        // Legendary SBA removes one duplicate only when priority next resumes.
        assert!(!g.keywords(other).contains(Keywords::INDESTRUCTIBLE));
        g.move_object(other, Zone::Exile);
        let terror = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, terror, None, &[Target::Card(id)]);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Divinity), 1);
        assert!(!g.keywords(id).contains(Keywords::INDESTRUCTIBLE));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::Activate { card, .. } if *card == id))
        );
    }
}
#[test]
fn fdn_bounded_core_myojin_free_hand_cast_and_nonhand_normal_cast_are_distinct() {
    for p in [Player::P0, Player::P1] {
        for origin in [Zone::Hand, Zone::Graveyard, Zone::Exile] {
            let mut g = setup(p);
            let id = gated(&mut g, p, "Myojin of Night's Reach", origin);
            // This directly tests the existing actual cast builder: zero payment
            // and Normal method are deliberately identical for all origins.
            let item = g.cast_item(p, id, [0; 6]);
            assert_eq!(item.cast.unwrap().origin, origin);
            g.state.stack.push(item);
            settle(&mut g);
            assert_eq!(
                g.counter_count(id, CounterKind::Divinity),
                u64::from(origin == Zone::Hand)
            );
            assert_eq!(
                g.keywords(id).contains(Keywords::INDESTRUCTIBLE),
                origin == Zone::Hand
            );
        }
    }
}
#[test]
fn fdn_bounded_core_myojin_paid_counter_removal_discard_and_owned_privacy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = hand_myojin(&mut g, p);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let ours = named(&mut g, p, "Bear Cub", Zone::Hand);
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Activate { card, .. } if *card == id))
            .unwrap();
        let mut invalid = action.clone();
        if let Action::Activate {
            counter_payment, ..
        } = &mut invalid
        {
            *counter_payment = Counters::of(CounterKind::PlusOne, 1);
        }
        reject(&mut g, invalid);
        checked(&mut g, action.clone());
        assert_eq!(g.counter_count(id, CounterKind::Divinity), 0);
        assert!(!g.keywords(id).contains(Keywords::INDESTRUCTIBLE));
        reject(&mut g, action);
        // The ability resolves independently after its source departs.
        g.move_object(id, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(g.object_view(ours).zone, Zone::Hand);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == ours));
    }
}
#[test]
fn fdn_bounded_core_venom_counts_resolutions_not_entries_and_exactly_second_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let v = venom(&mut g, p);
        let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert!(g.state.ability_resolutions.is_none());
        // Three simultaneous entries queue three triggers before any resolves.
        let a = named(&mut g, p, "Bear Cub", Zone::Hand);
        let b = named(&mut g, p, "Bear Cub", Zone::Hand);
        let c = named(&mut g, p, "Bear Cub", Zone::Hand);
        let moves = [a, b, c]
            .into_iter()
            .map(|id| g.entry_move(id, Zone::Battlefield, false))
            .collect();
        g.stage_entries(moves, None);
        until_stack(&mut g, 3);
        assert!(!g.keywords(v).contains(Keywords::DEATHTOUCH));
        until_stack(&mut g, 2);
        assert!(g.keywords(v).contains(Keywords::DEATHTOUCH));
        assert!(!g.keywords(ally).contains(Keywords::DEATHTOUCH));
        until_stack(&mut g, 1);
        assert!(g.keywords(ally).contains(Keywords::DEATHTOUCH));
        let late = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        settle(&mut g);
        assert!(!g.keywords(late).contains(Keywords::DEATHTOUCH));
        let views = g.observe(p).ability_resolutions;
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].count, 3);
        assert_eq!(views[0].source, g.object_ref(v));
        assert_eq!(
            serde_json::to_value(&views).unwrap(),
            serde_json::to_value(g.observe(p.other()).ability_resolutions).unwrap()
        );
        g.finish_cleanup();
        assert!(!g.keywords(ally).contains(Keywords::DEATHTOUCH));
        g.begin_turn();
        assert!(g.state.ability_resolutions.is_none());
    }
}
#[test]
fn fdn_bounded_core_venom_departed_source_packets_and_new_incarnation_do_not_mix() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let v = venom(&mut g, p);
        let a = named(&mut g, p, "Bear Cub", Zone::Hand);
        let b = named(&mut g, p, "Bear Cub", Zone::Hand);
        let moves = [a, b]
            .into_iter()
            .map(|id| g.entry_move(id, Zone::Battlefield, false))
            .collect();
        g.stage_entries(moves, None);
        until_stack(&mut g, 2);
        let old = g.object_ref(v);
        g.move_object(v, Zone::Exile);
        g.move_object(v, Zone::Battlefield);
        let new = g.object_ref(v);
        assert_ne!(new, old);
        // Counter one packet by removing it without resolving. It must not count.
        g.state.stack.pop().unwrap();
        settle(&mut g);
        assert!(!g.keywords(v).contains(Keywords::DEATHTOUCH));
        assert!(!g.keywords(a).contains(Keywords::DEATHTOUCH));
        assert_eq!(g.observe(p).ability_resolutions[0].source, old);
        assert_eq!(g.observe(p).ability_resolutions[0].count, 1);
        let d = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.move_object(d, Zone::Battlefield);
        settle(&mut g);
        assert!(g.keywords(v).contains(Keywords::DEATHTOUCH));
        assert!(!g.keywords(a).contains(Keywords::DEATHTOUCH));
        let rows = g.observe(p).ability_resolutions;
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|r| r.count == 1));
    }
}

#[test]
fn fdn_bounded_core_myojin_entry_counter_doubling_and_lethal_after_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let id = hand_myojin(&mut g, p);
        assert_eq!(g.counter_count(id, CounterKind::Divinity), 2);
        paid(&mut g, p, "Shock", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert_eq!(g.object_view(id).damage, 2);
        for remaining in [1, 0] {
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Activate { card, .. } if *card == id))
                .unwrap();
            checked(&mut g, action);
            if remaining == 1 {
                assert_eq!(g.counter_count(id, CounterKind::Divinity), 1);
                assert!(g.keywords(id).contains(Keywords::INDESTRUCTIBLE));
                settle(&mut g);
            } else {
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
                assert_eq!(g.counter_count(id, CounterKind::Divinity), 0);
                settle(&mut g);
            }
        }
    }
}
#[test]
fn fdn_bounded_core_venom_source_control_and_removal_leave_queued_resolution_independent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let v = venom(&mut g, p);
        let original_ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let a = named(&mut g, p, "Bear Cub", Zone::Hand);
        let b = named(&mut g, p, "Bear Cub", Zone::Hand);
        let moves = [a, b]
            .into_iter()
            .map(|id| g.entry_move(id, Zone::Battlefield, false))
            .collect();
        g.stage_entries(moves, None);
        until_stack(&mut g, 2);
        let packet = g.state.stack.last().unwrap().clone();
        g.gain_control(&packet, Subject::Source, PlayerSet::Opponent);
        settle(&mut g);
        assert!(g.keywords(v).contains(Keywords::DEATHTOUCH));
        assert!(g.keywords(original_ally).contains(Keywords::DEATHTOUCH));
        assert!(!g.keywords(enemy).contains(Keywords::DEATHTOUCH));
        let before = g.observe(p).ability_resolutions;
        paid(&mut g, p, "Stop Cold", &[Target::Card(v)]);
        let d = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
        g.move_object(d, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.observe(p).ability_resolutions, before);
        assert!(!g.keywords(d).contains(Keywords::DEATHTOUCH));
        assert_eq!(std::mem::size_of::<Object>(), 144);
    }
}

#[test]
fn fdn_eight_core_qualified_source_and_relational_target_groups_work_together() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elenda = gated(
            &mut g,
            p.other(),
            "Elenda, Saint of Dusk",
            Zone::Battlefield,
        );
        let knight = gated(&mut g, p.other(), "Knight of Grace", Zone::Battlefield);
        let bow = named(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        attach(&mut g, bow, knight);
        let fiery = gated(&mut g, p, "Fiery Annihilation", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: fiery });
        let prohibited = Action::ChooseTarget {
            target: Target::Card(elenda),
        };
        assert!(!g.legal_actions().contains(&prohibited));
        reject(&mut g, prohibited);
        // Red instant: Elenda is protected; hexproof from black does not protect the Knight.
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(knight),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bow),
            },
        );
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, a);
        }
        settle(&mut g);
        assert_eq!(g.object_view(elenda).zone, Zone::Battlefield);
        assert_eq!(g.object_view(knight).zone, Zone::Exile);
        assert_eq!(g.object_view(bow).zone, Zone::Exile);
    }
}
