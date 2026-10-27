//! Gated Hunter's Talent draft: Class levels and owned state.
use super::big_helpers::{checked, settle, setup};
use super::*;

// Bypass fixture admission solely for this still-gated candidate.
fn class(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Forest", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name("Hunter's Talent").unwrap();
    g.state.objects[id as usize].zone = Zone::Battlefield;
    g.register_battlefield_sources(id);
    id
}
fn activate(g: &mut Game, id: CardId, ability: u16) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    let payment = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, payment);
}
#[test]
fn class_levels_require_resolution_and_reset_on_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = class(&mut g, p);
        assert_eq!(g.object_view(id).class_level, Some(1));
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        super::contracts::branch_contract(g.clone());
        activate(&mut g, id, 0);
        assert_eq!(g.object_view(id).class_level, Some(1));
        settle(&mut g);
        assert_eq!(g.object_view(id).class_level, Some(2));
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        activate(&mut g, id, 1);
        settle(&mut g);
        assert_eq!(g.object_view(id).class_level, Some(3));
        g.move_object(id, Zone::Graveyard);
        g.move_object(id, Zone::Battlefield);
        assert_eq!(g.object_view(id).class_level, Some(1));
    }
}
#[test]
fn class_level_ability_does_not_change_a_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = class(&mut g, p);
        activate(&mut g, id, 0);
        g.move_object(id, Zone::Graveyard);
        g.move_object(id, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(id).class_level, Some(1));
    }
}

#[test]
fn class_end_step_requires_ferocious_at_trigger_and_resolution() {
    for p in [Player::P0, Player::P1] {
        for lose_creature in [false, true] {
            let mut g = setup(p);
            let id = class(&mut g, p);
            activate(&mut g, id, 0);
            settle(&mut g);
            activate(&mut g, id, 1);
            settle(&mut g);
            let large = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            g.state.objects[large as usize].growth.power = 1;
            let before = g.state.players[p.index()].cards_drawn_this_turn;
            g.begin_step(Step::End);
            if lose_creature {
                g.move_object(large, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn - before,
                u64::from(!lose_creature)
            );
        }
        let mut g = setup(p);
        let id = class(&mut g, p);
        activate(&mut g, id, 0);
        settle(&mut g);
        activate(&mut g, id, 1);
        settle(&mut g);
        let before = g.state.players[p.index()].cards_drawn_this_turn;
        g.begin_step(Step::End);
        put(&mut g, p, "Hill Giant", Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before);
    }
}

#[test]
fn class_level_belongs_to_permanent_not_copiable_characteristics() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = class(&mut g, p);
        activate(&mut g, id, 0);
        settle(&mut g);
        let other = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let recipe = CopyRecipe::new(Card::from_name("Hunter's Talent").unwrap());
        g.apply_temporary_copy(g.object_ref(other), recipe, CopyDeadline::EndOfTurn);
        assert_eq!(g.object_view(other).class_level, Some(1));
        activate(&mut g, other, 0);
        settle(&mut g);
        assert_eq!(g.object_view(other).class_level, Some(2));
        let bear = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        g.apply_temporary_copy(g.object_ref(id), bear, CopyDeadline::EndOfTurn);
        assert_eq!(g.object_view(id).class_level, Some(2));
        super::contracts::branch_contract(g.clone());
    }
}

#[test]
fn class_entry_damage_uses_two_independent_legal_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let friendly = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let id = put(&mut g, p, "Forest", Zone::Hand);
        g.state.objects[id as usize].card = Card::from_name("Hunter's Talent").unwrap();
        g.move_object(id, Zone::Battlefield);
        g.prepare_triggers();
        for _ in 0..12 {
            if matches!(g.state.pending, Pending::Targets { .. }) {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .unwrap();
            checked(&mut g, action);
        }
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(friendly),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(friendly),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(enemy).damage, 2);
    }
}

#[test]
fn class_level_two_grants_one_attack_trigger_for_the_declaration() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = class(&mut g, p);
        activate(&mut g, id, 0);
        settle(&mut g);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: a });
        checked(&mut g, Action::Attack { card: b });
        checked(&mut g, Action::FinishAttackers);
        for _ in 0..16 {
            if matches!(g.state.pending, Pending::Targets { .. }) {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .unwrap();
            checked(&mut g, action);
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        for _ in 0..16 {
            if g.keywords(a).contains(Keywords::TRAMPLE) {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .unwrap();
            checked(&mut g, action);
        }
        assert_eq!(g.stats(a).power, 3);
        assert!(g.keywords(a).contains(Keywords::TRAMPLE));
        assert_eq!(g.stats(b).power, 2);
    }
}

#[test]
fn class_activations_require_control_and_sorcery_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = class(&mut g, p);
        let action = Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        };
        assert!(g.legal_actions().contains(&action));
        g.state.step = Step::BeginCombat;
        assert!(!g.legal_actions().contains(&action));
        g.state.step = Step::Main1;
        g.state.priority = p.other();
        assert!(!g.legal_actions().contains(&action));
        g.state.objects[id as usize].controller = p.other();
        g.state.active = p.other();
        assert!(g.legal_actions().contains(&action));
        checked(&mut g, action);
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        settle(&mut g);
        assert_eq!(g.object_view(id).class_level, Some(2));
    }
}
#[test]
fn hunter_paid_cast_entry_respects_changed_target_control() {
    for p in [Player::P0, Player::P1] {
        for change in [None, Some(0), Some(1)] {
            let mut g = setup(p);
            let friendly = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let id = put(&mut g, p, "Forest", Zone::Hand);
            g.state.objects[id as usize].card = Card::from_name("Hunter's Talent").unwrap();
            let mana_before = g.state.players[p.index()].mana;
            big_helpers::cast(&mut g, id, None, &[]);
            assert_ne!(g.state.players[p.index()].mana, mana_before);
            for _ in 0..30 {
                if matches!(g.state.pending, Pending::Targets { .. }) {
                    break;
                }
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                    .unwrap();
                checked(&mut g, action);
            }
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(friendly),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            match change {
                Some(0) => g.record_control(g.object_ref(friendly), p.other(), None),
                Some(1) => g.record_control(g.object_ref(enemy), p, None),
                _ => (),
            }
            settle(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(enemy).damage,
                if change.is_none() { 2 } else { 0 }
            );
        }
    }
}

#[test]
fn countered_class_level_activation_does_not_advance() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = class(&mut g, p);
        activate(&mut g, id, 0);
        let packet = g.state.stack.last().unwrap().id;
        assert!(g.counter_stack_item(packet, false));
        settle(&mut g);
        assert_eq!(g.object_view(id).class_level, Some(1));
        assert!(g.legal_actions().contains(&Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        }));
    }
}

#[test]
fn pending_class_level_ability_sets_level_after_source_stops_being_a_class() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = class(&mut g, p);
        activate(&mut g, id, 0);
        g.apply_temporary_copy(
            g.object_ref(id),
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert!(!g.has_subtype(id, "Class"));
        settle(&mut g);
        assert_eq!(g.object_view(id).class_level, Some(2));
        g.finish_cleanup();
        assert!(g.has_subtype(id, "Class"));
        assert_eq!(g.object_view(id).class_level, Some(2));
    }
}

#[test]
fn queued_ferocious_trigger_survives_source_departure_or_ability_loss() {
    for p in [Player::P0, Player::P1] {
        for remove in [true, false] {
            let mut g = setup(p);
            let id = class(&mut g, p);
            activate(&mut g, id, 0);
            settle(&mut g);
            activate(&mut g, id, 1);
            settle(&mut g);
            let creature = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            g.state.objects[creature as usize].growth.power = 1;
            let before = g.state.players[p.index()].cards_drawn_this_turn;
            g.begin_step(Step::End);
            g.give_priority();
            g.prepare_triggers();
            if remove {
                g.move_object(id, Zone::Graveyard);
            } else {
                g.apply_temporary_copy(
                    g.object_ref(id),
                    CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
                    CopyDeadline::EndOfTurn,
                );
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 1);
        }
    }
}

#[test]
fn queued_attack_trigger_survives_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = class(&mut g, p);
        activate(&mut g, id, 0);
        settle(&mut g);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: attacker });
        checked(&mut g, Action::FinishAttackers);
        for _ in 0..16 {
            if matches!(g.state.pending, Pending::Targets { .. }) {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .unwrap();
            checked(&mut g, action);
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(attacker),
            },
        );
        g.move_object(id, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.stats(attacker).power, 3);
        assert!(g.keywords(attacker).contains(Keywords::TRAMPLE));
    }
}
