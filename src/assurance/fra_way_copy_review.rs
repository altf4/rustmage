use super::big_helpers::{cast, checked, settle, setup};
use super::*;

fn setup_way(g: &mut Game, p: Player, name: &str) -> (CardId, CardId) {
    let way = put(g, p, "Forest", Zone::Hand);
    g.state.objects[way as usize].card = Card::from_name(name).unwrap();
    cast(g, way, None, &[]);
    settle(g);
    let walker = g
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(i, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(i as CardId, "Jace"))
            .then_some(i as CardId)
        })
        .unwrap();
    (way, walker)
}

#[test]
fn copied_way_provider_gives_and_loses_live_grant() {
    for p in [Player::P0, Player::P1] {
        for name in ["Way of the Healer", "Way of the Pyromancer"] {
            let mut g = setup(p);
            let (way, walker) = setup_way(&mut g, p, name);
            g.move_object(way, Zone::Graveyard);
            let copy = put(&mut g, p, "Phyrexian Arena", Zone::Battlefield);
            let reference = g.object_ref(copy);
            let action = Action::AnnouncePlaneswalkerGrant {
                source: g.object_ref(walker),
                grantor: reference,
                ability: 0,
            };
            assert!(!g.legal_actions().contains(&action));
            g.apply_temporary_copy(
                reference,
                CopyRecipe::new(Card::from_name(name).unwrap()),
                CopyDeadline::EndOfTurn,
            );
            assert!(
                g.legal_actions().contains(&action),
                "copying {name} should grant its ability"
            );
            super::contracts::branch_contract(g.clone());
            g.apply_temporary_copy(
                reference,
                CopyRecipe::new(Card::from_name("Phyrexian Arena").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            assert!(
                !g.legal_actions().contains(&action),
                "copying away from {name} should revoke its ability"
            );
        }
    }
}

#[test]
fn grant_requires_live_planeswalker_type_and_returns_when_copy_expires() {
    for p in [Player::P0, Player::P1] {
        for name in ["Way of the Healer", "Way of the Pyromancer"] {
            let mut g = setup(p);
            let (way, walker) = setup_way(&mut g, p, name);
            let reference = g.object_ref(walker);
            let action = Action::AnnouncePlaneswalkerGrant {
                source: reference,
                grantor: g.object_ref(way),
                ability: 0,
            };
            assert!(g.legal_actions().contains(&action));
            g.apply_temporary_copy(
                reference,
                CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            assert_eq!(g.current_kinds(walker) & 64, 0);
            assert!(!g.legal_actions().contains(&action));
            g.finish_cleanup();
            g.state.active = p;
            g.state.priority = p;
            g.state.step = Step::Main1;
            assert_ne!(g.current_kinds(walker) & 64, 0);
            assert!(g.legal_actions().contains(&action));
        }
    }
}

#[test]
fn pyromancer_stack_source_is_walker_and_mana_survives_both_departures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (way, walker) = setup_way(&mut g, p, "Way of the Pyromancer");
        let reference = g.object_ref(walker);
        let way_ref = g.object_ref(way);
        let action = Action::AnnouncePlaneswalkerGrant {
            source: reference,
            grantor: way_ref,
            ability: 0,
        };
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, action);
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        let packet = g.state.stack.last().unwrap();
        assert_eq!(packet.object, reference);
        assert_eq!(packet.card, walker);
        assert_eq!(packet.source_facts.card, g.object_view(walker).card);
        assert_eq!(packet.controller, p);
        let snapshot = g.snapshot();
        super::contracts::branch_contract(g.clone());
        g.move_object(way, Zone::Graveyard);
        g.move_object(walker, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
        g.restore(&snapshot).unwrap();
        g.move_object(walker, Zone::Exile);
        g.move_object(way, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
    }
}
