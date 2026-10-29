//! Exact granting incarnations, alternate faces, and genuinely paid additional costs.
use super::big_helpers::{cast as paid, checked, next_turn, settle as finish};
use super::*;
fn action(card: CardId, permission: ObjectRef, face: CastFace) -> Action {
    Action::AnnounceFreeFromPermission {
        card,
        permission,
        face,
        flash_surcharge: false,
    }
}
fn zero_pay(g: &mut Game) {
    for _ in 0..6 {
        if g.state.announcing.is_none() {
            return;
        }
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(g, Action::FinishTargets);
        } else {
            checked(g, Action::PayCosts { payment: [0; 6] });
        }
    }
    panic!("unpaid free cast");
}
#[test]
fn sos_zaffai_paid_body_own_turn_hand_types_and_exact_once_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Zaffai and the Tempests", Zone::Hand);
        paid(&mut g, source, None, &[]);
        finish(&mut g);
        assert_eq!(stats(&g, source), (5, 7));
        let permission = g.object_ref(source);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        let second = put(&mut g, p, "Shock", Zone::Hand);
        let creature = put(&mut g, p, "Bear Cub", Zone::Hand);
        let land = put(&mut g, p, "Island", Zone::Hand);
        let grave = put(&mut g, p, "Shock", Zone::Graveyard);
        let foreign = put(&mut g, p.other(), "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        for card in [creature, land, grave, foreign] {
            reject(&mut g, action(card, permission, CastFace::Normal));
        }
        let wrong = g.object_ref(creature);
        reject(&mut g, action(shock, wrong, CastFace::Normal));
        checked(&mut g, action(shock, permission, CastFace::Normal));
        assert_eq!(
            g.observe(p.other()).announcing_free_hand_permission,
            Some(permission)
        );
        assert!(g.observe(p).used_free_hand_permissions.is_empty());
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        zero_pay(&mut g);
        let spell = g.state.stack.iter().find(|s| s.source.is_some_and(|source| source.id == shock)).unwrap();
        assert_eq!(spell.cast.unwrap().payment, [0; 6]);
        assert_eq!(spell.mana_value(), 1);
        assert_eq!(
            g.observe(p.other()).used_free_hand_permissions,
            vec![permission]
        );
        finish(&mut g);
        reject(&mut g, action(second, permission, CastFace::Normal));
        next_turn(&mut g);
        reject(&mut g, action(second, permission, CastFace::Normal));
        next_turn(&mut g);
        checked(&mut g, action(second, permission, CastFace::Normal));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        zero_pay(&mut g);
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}
#[test]
fn sos_zaffai_new_incarnation_is_independent_and_omniscience_does_not_spend_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Zaffai and the Tempests", Zone::Battlefield);
        put(&mut g, p, "Omniscience", Zone::Battlefield);
        let old = g.object_ref(source);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        checked(
            &mut g,
            Action::AnnounceFree {
                card: shock,
                face: CastFace::Normal,
                flash_surcharge: false,
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        zero_pay(&mut g);
        finish(&mut g);
        assert!(g.observe(p).used_free_hand_permissions.is_empty());
        let draw = put(&mut g, p, "Divination", Zone::Hand);
        checked(&mut g, action(draw, old, CastFace::Normal));
        zero_pay(&mut g);
        finish(&mut g);
        let blink = put(&mut g, p, "Daydream", Zone::Hand);
        paid(&mut g, blink, None, &[Target::Card(source)]);
        finish(&mut g);
        let new = g.object_ref(source);
        assert_ne!(old, new);
        let draw = put(&mut g, p, "Divination", Zone::Hand);
        reject(&mut g, action(draw, old, CastFace::Normal));
        checked(&mut g, action(draw, new, CastFace::Normal));
        zero_pay(&mut g);
        finish(&mut g);
        assert_eq!(g.observe(p).used_free_hand_permissions, vec![old, new]);
    }
}
#[test]
fn sos_zaffai_permission_cost_survives_grantor_sacrifice_and_requires_actual_additional_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Zaffai and the Tempests", Zone::Battlefield);
        let permission = g.object_ref(source);
        let spell = put(&mut g, p, "Corrupted Conviction", Zone::Hand);
        let hand = g.hand_count(p) - 1;
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, action(spell, permission, CastFace::Normal));
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        checked(&mut g, Action::ChooseAdditionalCost { object: permission });
        zero_pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.hand_count(p), hand + 2);
        assert_eq!(g.observe(p).used_free_hand_permissions, vec![permission]);
    }
}
#[test]
fn sos_zaffai_alternate_spell_face_and_mana_x_zero_preserve_normal_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Zaffai and the Tempests", Zone::Battlefield);
        let permission = g.object_ref(source);
        let alchemist = put(&mut g, p, "Aquatic Alchemist", Zone::Hand);
        let grave = put(&mut g, p, "Shock", Zone::Graveyard);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, action(alchemist, permission, CastFace::Normal));
        checked(&mut g, action(alchemist, permission, CastFace::Adventure));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(grave),
            },
        );
        zero_pay(&mut g);
        assert_eq!(g.state.stack.last().unwrap().mana_value(), 3);
        finish(&mut g);
        assert_eq!(g.object_view(alchemist).zone, Zone::Exile);
        let mut g = scenario(p);
        let source = put(&mut g, p, "Zaffai and the Tempests", Zone::Battlefield);
        let permission = g.object_ref(source);
        let spell = put(&mut g, p, "Mind into Matter", Zone::Hand);
        g.state.step = Step::BeginCombat;
        reject(&mut g, action(spell, permission, CastFace::Normal));
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, action(spell, permission, CastFace::Normal));
        reject(&mut g, Action::ChooseX { value: 1 });
        zero_pay(&mut g);
        let stack = g.state.stack.iter().find(|s| s.source.is_some_and(|source| source.id == spell)).unwrap();
        assert_eq!(stack.x, Some(0));
        assert_eq!(stack.mana_value(), 2);
        finish(&mut g);
    }
}
