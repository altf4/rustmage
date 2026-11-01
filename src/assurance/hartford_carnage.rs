//! Mayhem's exact discard origin and the returned creature's independent abilities.
use super::big_helpers::{cast, checked, settle, setup};
use super::*;

fn reach_target(g: &mut Game) {
    for _ in 0..30 {
        if matches!(g.state.pending, Pending::Targets { .. }) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            .unwrap();
        checked(g, action);
    }
    panic!("missing Carnage target");
}
fn discard_with_thrill(g: &mut Game, p: Player, card: CardId) {
    let spell = put(g, p, "Thrill of Possibility", Zone::Hand);
    checked(g, Action::AnnounceCast { card: spell });
    for _ in 0..10 {
        if g.legal_actions().contains(&Action::Discard { card }) {
            break;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::ChooseCastCost { .. }))
            .unwrap();
        checked(g, action);
    }
    checked(g, Action::Discard { card });
    while g.state.announcing.is_some() {
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::ChooseCastCost { .. }))
            .unwrap();
        checked(g, action);
    }
    settle(g);
}

#[test]
fn carnage_returns_a_qualifying_owned_creature_and_grants_independent_combat_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let recipient = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let large = put(&mut g, p, "Serra Angel", Zone::Graveyard);
        let source = put(&mut g, p, "Carnage, Crimson Chaos", Zone::Hand);
        cast(&mut g, source, None, &[]);
        reach_target(&mut g);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(large),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(recipient),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(recipient).zone, Zone::Battlefield);
        assert!(g.attacks_each_combat(recipient));
        assert!(g.keywords(source).contains(Keywords::TRAMPLE));
        g.move_object(source, Zone::Graveyard);
        g.state.objects[recipient as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::FinishAttackers);
        checked(&mut g, Action::Attack { card: recipient });
        checked(&mut g, Action::FinishAttackers);
        let life = g.state.players[p.other().index()].life;
        for _ in 0..60 {
            if g.object_view(recipient).zone == Zone::Graveyard {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass | Action::FinishBlockers | Action::PutTrigger { .. }
                    )
                })
                .unwrap();
            checked(&mut g, action);
        }
        assert_eq!(g.object_view(recipient).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
        g.move_object(recipient, Zone::Battlefield);
        assert!(!g.attacks_each_combat(recipient));
    }
}

#[test]
fn carnage_mayhem_pays_br_after_actual_discard_and_countering_returns_it_to_the_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = put(&mut g, p, "Carnage, Crimson Chaos", Zone::Hand);
        discard_with_thrill(&mut g, p, source);
        assert_eq!(
            g.observe(p.other()).mayhem_discards_this_turn,
            vec![g.object_ref(source)]
        );
        g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 0];
        g.state.active = p.other();
        g.state.priority = p;
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceMayhem { card: source })
        );
        g.state.active = p;
        checked(&mut g, Action::AnnounceMayhem { card: source });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Mayhem
        );
        checked(&mut g, Action::Pass);
        let counter = put(&mut g, p.other(), "Cancel", Zone::Hand);
        cast(&mut g, counter, None, &[Target::Spell(source)]);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert!(!g.can_cast_mayhem(p, source));
        assert!(g.observe(p).mayhem_discards_this_turn.is_empty());
    }
}

#[test]
fn carnage_mayhem_does_not_remember_a_new_incarnation_or_a_later_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = put(&mut g, p, "Carnage, Crimson Chaos", Zone::Graveyard);
        assert!(!g.can_cast_mayhem(p, source));
        g.move_object(source, Zone::Hand);
        g.discard_batch(&[source], None);
        assert!(g.can_cast_mayhem(p, source));
        assert!(!g.can_cast_mayhem(p.other(), source));
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Graveyard);
        assert!(!g.can_cast_mayhem(p, source));
        g.move_object(source, Zone::Hand);
        g.discard_batch(&[source], None);
        assert!(g.can_cast_mayhem(p, source));
        g.state.turn += 1;
        g.begin_turn();
        assert!(!g.can_cast_mayhem(p, source));
    }
}
