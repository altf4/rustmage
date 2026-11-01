//! Bitter Triumph: mutually exclusive additional costs and atomic announcement.
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};

fn begin_additional(g: &mut Game) {
    big_helpers::checked(g, Action::BeginOrderedCosts);
    let index = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == CastCostClause::Additional)
        .unwrap() as u16;
    big_helpers::checked(g, Action::ChooseCastCost { clause: index });
}
fn finish_mana(g: &mut Game) {
    let index = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == CastCostClause::Mana)
        .unwrap() as u16;
    big_helpers::checked(g, Action::ChooseCastCost { clause: index });
    pay(g);
}

#[test]
fn bitter_triumph_life_and_discard_routes_are_private_and_paid_once() {
    for p in [Player::P0, Player::P1] {
        for pay_life in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Bitter Triumph", Zone::Hand);
            let discard = put(&mut g, p, "Island", Zone::Hand);
            let foreign = put(&mut g, p.other(), "Island", Zone::Hand);
            let life = g.state.players[p.index()].life;
            big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
            big_helpers::checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            begin_additional(&mut g);
            let object = g.object_ref(foreign);
            reject(&mut g, Action::ChooseAdditionalCost { object });
            assert!(g.observe(p.other()).additional_cost.is_none());
            contracts::branch_contract(g.clone());
            if pay_life {
                big_helpers::checked(&mut g, Action::ChooseAdditionalLife);
            } else {
                let object = g.object_ref(discard);
                big_helpers::checked(&mut g, Action::ChooseAdditionalCost { object });
            }
            finish_mana(&mut g);
            big_helpers::settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                life - if pay_life { 3 } else { 0 }
            );
            assert_eq!(
                g.object_view(discard).zone,
                if pay_life {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn bitter_triumph_low_life_requires_another_hand_card() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Bitter Triumph", Zone::Hand);
        g.state.players[p.index()].life = 2;
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        let discard = put(&mut g, p, "Island", Zone::Hand);
        let target = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.controller == p.other())
            .map(|(i, _)| i as CardId)
            .unwrap();
        big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        begin_additional(&mut g);
        reject(&mut g, Action::ChooseAdditionalLife);
        let object = g.object_ref(discard);
        big_helpers::checked(&mut g, Action::ChooseAdditionalCost { object });
        finish_mana(&mut g);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 2);
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
    }
}

#[test]
fn bitter_triumph_funding_preserves_returned_discard_and_branching() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.state.players[p.index()].life = 2;
        g.state.players[p.index()].mana = [0; 6];
        put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Bitter Triumph", Zone::Hand);
        let bolg = put(&mut g, p, "Bolg's Company", Zone::Battlefield);
        let goblin = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let discard = put(&mut g, p, "Island", Zone::Exile);
        let before = g.snapshot();
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        reject(&mut g, Action::AnnounceCast { card: spell });
        assert_eq!(g.snapshot(), before);

        // Sacrificing the linked Goblin for Bolg's mana returns a card to hand.
        // Funding must explore that line before rejecting the additional cost.
        g.state.exile_links.push(ExileLink {
            source: g.object_ref(goblin),
            exiled: g.object_ref(discard),
            return_zone: Zone::Hand,
        });
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        contracts::branch_contract(g.clone());
        assert_eq!(g.object_view(bolg).zone, Zone::Battlefield);
        assert_eq!(g.object_view(discard).zone, Zone::Exile);
    }
}

#[test]
fn bitter_triumph_can_pay_mana_before_life_route() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Bitter Triumph", Zone::Hand);
        let life = g.state.players[p.index()].life;
        big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        let index = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|r| r.clause == CastCostClause::Additional)
            .unwrap() as u16;
        big_helpers::checked(&mut g, Action::ChooseCastCost { clause: index });
        big_helpers::checked(&mut g, Action::ChooseAdditionalLife);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life - 3);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    }
}

#[test]
fn bitter_triumph_can_pay_exactly_three_life() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Bitter Triumph", Zone::Hand);
        g.state.players[p.index()].life = 3;
        big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        begin_additional(&mut g);
        big_helpers::checked(&mut g, Action::ChooseAdditionalLife);
        assert_eq!(g.state.players[p.index()].life, 0);
        finish_mana(&mut g);
    }
}

#[test]
fn bitter_triumph_lost_target_still_costs_and_does_not_hit_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Bitter Triumph", Zone::Hand);
        let life = g.state.players[p.index()].life;
        big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        begin_additional(&mut g);
        big_helpers::checked(&mut g, Action::ChooseAdditionalLife);
        finish_mana(&mut g);
        g.move_object(target, Zone::Hand);
        g.move_object(target, Zone::Battlefield);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].life, life - 3);
    }
}

#[test]
fn bitter_triumph_funding_accounts_for_life_spent_on_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Great Hall of the Biblioplex", Zone::Battlefield);
        let spell = put(&mut g, p, "Bitter Triumph", Zone::Hand);
        g.state.players[p.index()].life = 3;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, Action::AnnounceCast { card: spell });
        put(&mut g, p, "Island", Zone::Hand);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
    }
}

#[test]
fn bitter_triumph_can_be_cast_from_exile_with_additional_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Bitter Triumph", Zone::Exile);
        let object = g.object_ref(spell);
        g.grant_exile_play_permission(p, object, false);
        let life = g.state.players[p.index()].life;
        big_helpers::checked(&mut g, Action::AnnounceExile { card: spell });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        begin_additional(&mut g);
        big_helpers::checked(&mut g, Action::ChooseAdditionalLife);
        finish_mana(&mut g);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, life - 3);
    }
}

#[test]
fn bitter_triumph_partial_ordered_cost_restore_recovers_unpaid_state() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Bitter Triumph", Zone::Hand);
        big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        begin_additional(&mut g);
        let before = g.snapshot();
        let life = g.state.players[p.index()].life;
        let mut branch = g.clone();
        big_helpers::checked(&mut branch, Action::ChooseAdditionalLife);
        assert_eq!(branch.state.players[p.index()].life, life - 3);
        branch.restore(&before).unwrap();
        assert_eq!(branch.snapshot(), before);
        assert_eq!(branch.state.players[p.index()].life, life);
        assert!(
            branch
                .legal_actions()
                .contains(&Action::ChooseAdditionalLife)
        );
        assert_eq!(g.snapshot(), before);
    }
}

#[test]
fn bitter_triumph_foreign_choice_rolls_back_partially_paid_cost() {
    for (p, pay_life, exile_discard, shuffle_discard) in [
        (Player::P0, true, false, false),
        (Player::P1, true, false, false),
        (Player::P0, false, false, false),
        (Player::P1, false, false, false),
        (Player::P0, false, true, false),
        (Player::P1, false, true, false),
        (Player::P0, false, false, true),
        (Player::P1, false, false, true),
    ] {
        let mut g = scenario(p);
        let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        g.state.objects[silver as usize].growth.power = -4;
        let copy = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let mut recipe = CopyRecipe::new(Card::from_name("Silverquill, the Disputant").unwrap());
        recipe.overrides = Some(CopyOverrides {
            legendary: Some(false),
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(g.object_ref(copy), recipe, CopyDeadline::EndOfTurn);
        g.state.objects[copy as usize].growth.power = -4;
        let candidate = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[candidate as usize].growth.power = -1;
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(enemy));
        let prison = put(&mut g, p, "Banishing Light", Zone::Hand);
        big_helpers::cast(&mut g, prison, None, &[]);
        for _ in 0..40 {
            if g.state.pending != Pending::Priority {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(aura),
            },
        );
        big_helpers::settle(&mut g);
        g.change_object_form(
            g.object_ref(prison),
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((1, 1)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        if exile_discard {
            put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
        }
        let spell = put(&mut g, p, "Bitter Triumph", Zone::Hand);
        let discard = put(
            &mut g,
            p,
            if shuffle_discard {
                "Darksteel Colossus"
            } else {
                "Island"
            },
            Zone::Hand,
        );
        let secret = put(&mut g, p, "Mountain", Zone::Hand);
        let revealed = put(&mut g, p, "Plains", Zone::Hand);
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == discard || o.id == secret)
        );
        let life = g.state.players[p.index()].life;
        big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
        big_helpers::checked(&mut g, Action::ChooseCasualty { pay: true });
        big_helpers::checked(&mut g, Action::ChooseCasualty { pay: true });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        // Exercise the same bookkeeping used by public reveal-in-hand effects.
        let revealed_ref = g.object_ref(revealed);
        g.remember_public_hand(revealed_ref);
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == revealed)
        );
        begin_additional(&mut g);
        if pay_life {
            big_helpers::checked(&mut g, Action::ChooseAdditionalLife);
            assert_eq!(g.state.players[p.index()].life, life - 3);
        } else {
            let object = g.object_ref(discard);
            big_helpers::checked(&mut g, Action::ChooseAdditionalCost { object });
            let destination = if exile_discard {
                Zone::Exile
            } else if shuffle_discard {
                Zone::Library
            } else {
                Zone::Graveyard
            };
            assert_eq!(g.object_view(discard).zone, destination);
            let opponent = g.observe(p.other());
            assert_eq!(
                opponent
                    .objects
                    .iter()
                    .any(|o| o.id == discard && o.zone == destination),
                !shuffle_discard
            );
            assert!(!opponent.objects.iter().any(|o| o.id == secret));
            contracts::branch_contract(g.clone());
        }
        let index = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|r| matches!(r.clause, CastCostClause::Casualty(_)) && !r.paid)
            .unwrap() as u16;
        big_helpers::checked(&mut g, Action::ChooseCastCost { clause: index });
        let object = g.object_ref(prison);
        big_helpers::checked(&mut g, Action::PayCasualty { object });
        assert_eq!(g.actor(), Some(p.other()));
        assert!(
            g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseAuraAttachment { .. }))
        );
        let object = g.object_ref(candidate);
        big_helpers::checked(&mut g, Action::ChooseAuraAttachment { object });
        assert!(g.state.cast_rollback.is_none());
        assert_eq!(g.state.players[p.index()].life, life);
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
        assert_eq!(
            g.object_view(discard).zone,
            if shuffle_discard {
                Zone::Library
            } else {
                Zone::Hand
            }
        );
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == revealed && o.zone == Zone::Hand)
        );
        if !pay_life && !shuffle_discard {
            assert!(
                g.observe(p.other())
                    .objects
                    .iter()
                    .any(|o| o.id == discard && o.zone == Zone::Hand)
            );
        }
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == secret));
    }
}

#[test]
fn rollback_does_not_publicize_face_down_exile_from_hand() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = put(&mut g, p, "Island", Zone::Hand);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.begin_attack_rollback(p);
        g.move_object(card, Zone::Exile);
        let exiled = g.object_ref(card);
        let source_ref = g.object_ref(source);
        g.state.linked_choices.get_or_insert_default().hidden.push((
            source_ref,
            exiled,
            1 << p.index(),
        ));
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == card));
        g.move_object(card, Zone::Hand);
        g.finish_cast_rollback(false);
        assert_eq!(g.object_view(card).zone, Zone::Hand);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == card));
    }
}

#[test]
fn rollback_remembers_public_exile_before_same_action_move() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = put(&mut g, p, "Island", Zone::Hand);
        g.begin_attack_rollback(p);
        g.move_object(card, Zone::Exile);
        // No action or observation boundary between the two zone moves.
        g.move_object(card, Zone::Hand);
        g.finish_cast_rollback(false);
        assert_eq!(g.object_view(card).zone, Zone::Hand);
        assert!(g.observe(p.other()).objects.iter().any(|o| o.id == card));
    }
}

#[test]
fn rollback_does_not_track_hand_identity_through_hidden_library() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = put(&mut g, p, "Island", Zone::Hand);
        let other_known = put(&mut g, p, "Mountain", Zone::Hand);
        let reference = g.object_ref(card);
        let other_ref = g.object_ref(other_known);
        g.remember_public_hand(reference);
        g.remember_public_hand(other_ref);
        assert!(g.observe(p.other()).objects.iter().any(|o| o.id == card));
        g.begin_attack_rollback(p);
        g.move_object(card, Zone::Library);
        g.state.rng.shuffle(&mut g.state.players[p.index()].library);
        g.move_object(card, Zone::Exile);
        g.prepare_cast_rollback(p, &Action::Pass);
        let rollback = g.state.cast_rollback.as_ref().unwrap();
        assert!(rollback.obscured_hand_cards.contains(&card));
        assert!(!rollback.public_hand_cards.contains(&card));
        g.finish_cast_rollback(false);
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == other_known)
        );
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == card && o.zone == Zone::Hand)
        );
    }
}
