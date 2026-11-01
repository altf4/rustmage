//! Additional reanimation and graveyard-origin interaction checks.
use super::big_helpers::{cast, checked, settle, setup};
use super::*;

fn candidate(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let id = put(g, p, "Forest", zone);
    g.state.objects[id as usize].card = Card::from_name(name).unwrap();
    id
}

fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, action);
}

fn cadets(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && o.card == Card::test_index(5298))
                .then_some(i as CardId)
        })
        .collect()
}

#[test]
fn rewrite_regrets_reanimates_eligible_planeswalker_and_empowers_jace() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Graveyard);
        let noncreature = put(&mut g, p, "Adventuring Gear", Zone::Graveyard);
        let spell = candidate(&mut g, p, "Rewrite Regrets", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(noncreature),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(walker),
            },
        );
        pay(&mut g);
        let checkpoint = g.snapshot();
        settle(&mut g);
        assert!(g.on_battlefield(walker));
        assert_eq!(g.object_view(walker).owner, p);
        assert_eq!(g.object_view(walker).controller, p);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 4);
        assert!(
            g.state
                .objects
                .iter()
                .enumerate()
                .any(|(i, o)| o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && g.has_subtype(i as CardId, "Jace")
                    && g.counter_count(i as CardId, CounterKind::Loyalty) == 2)
        );
        let complete = g.snapshot();
        g.restore(&checkpoint).unwrap();
        settle(&mut g);
        assert_eq!(g.snapshot(), complete);
    }
}

#[test]
fn recursive_recruitment_uses_graveyard_count_at_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for _ in 0..2 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        let extra = put(&mut g, p, "Forest", Zone::Hand);
        let spell = candidate(&mut g, p, "Recursive Recruitment", Zone::Graveyard);
        checked(&mut g, Action::AnnounceFlashback { card: spell });
        pay(&mut g);
        g.move_object(extra, Zone::Graveyard);
        let checkpoint = g.snapshot();
        settle(&mut g);
        let tokens = cadets(&g, p);
        assert_eq!(tokens.len(), 2);
        for id in tokens {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
        let complete = g.snapshot();
        g.restore(&checkpoint).unwrap();
        settle(&mut g);
        assert_eq!(g.snapshot(), complete);
    }
}

#[test]
fn recursive_recruitment_spell_copy_was_not_cast_from_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for _ in 0..3 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        let spell = candidate(&mut g, p, "Recursive Recruitment", Zone::Graveyard);
        checked(&mut g, Action::AnnounceFlashback { card: spell });
        pay(&mut g);
        let original = g
            .state
            .stack
            .iter()
            .find(|item| item.source.unwrap().id == spell)
            .unwrap()
            .clone();
        let copy = g.allocate_spell_copy(&original, p);
        g.state.stack.push(copy);
        assert!(g.counter_stack_item(original.id, false));
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        settle(&mut g);
        let tokens = cadets(&g, p);
        assert_eq!(tokens.len(), 2);
        for id in tokens {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
        }
    }
}

#[test]
fn countered_flashback_revival_goes_to_exile_and_does_not_return_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let spell = candidate(&mut g, p, "Generous Revival", Zone::Graveyard);
        let negate = put(&mut g, p.other(), "Negate", Zone::Hand);
        checked(&mut g, Action::AnnounceFlashback { card: spell });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        g.state.priority = p.other();
        cast(&mut g, negate, None, &[Target::Spell(spell)]);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
    }
}
