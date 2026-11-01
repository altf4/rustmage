//! Focused printed outcomes; shared engine invariants live in core tests.
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    }
}
