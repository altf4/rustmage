//! Graveyard landfall, life-conditioned haste and return as an enchantment.
use super::big_helpers::{checked, settle};
use super::*;

#[test]
fn bloodghast_graveyard_landfall_choice_and_live_haste() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Bloodghast", Zone::Graveyard);
            let land = put(&mut g, p, "Forest", Zone::Hand);
            let before = g.snapshot();
            checked(&mut g, Action::PlayLand { card: land });
            for _ in 0..30 {
                if g.legal_actions()
                    .contains(&Action::ChooseOptional { accept })
                {
                    break;
                }
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                    .unwrap();
                checked(&mut g, action);
            }
            checked(&mut g, Action::ChooseOptional { accept });
            settle(&mut g);
            if !accept {
                assert_eq!(g.object_view(source).zone, Zone::Graveyard);
                continue;
            }
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(g.object_view(source).controller, p);
            assert_eq!(stats(&g, source), (2, 1));
            assert!(g.keywords(source).contains(Keywords::CANT_BLOCK));
            g.state.players[p.other().index()].life = 11;
            assert!(!g.keywords(source).contains(Keywords::HASTE));
            g.state.players[p.other().index()].life = 10;
            assert!(g.keywords(source).contains(Keywords::HASTE));
            g.state.players[p.other().index()].life = 11;
            assert!(!g.keywords(source).contains(Keywords::HASTE));
            let mut branch = g.clone();
            branch.restore(&before).unwrap();
            assert_eq!(branch.object_view(source).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn bloodghast_opponent_land_and_stale_graveyard_incarnation_do_not_return_it() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bloodghast", Zone::Graveyard);
        let foreign = put(&mut g, p.other(), "Forest", Zone::Hand);
        g.state.active = p.other();
        g.state.priority = p.other();
        checked(&mut g, Action::PlayLand { card: foreign });
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        g.state.active = p;
        g.state.priority = p;
        g.state.land_played = false;
        let land = put(&mut g, p, "Forest", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: land });
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
}

#[test]
fn curiosity_dies_under_foreign_control_returns_to_owner_as_enchantment_and_keeps_identity() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Enduring Curiosity", Zone::Battlefield);
        g.state.objects[source as usize].controller = p.other();
        let old = g.object_ref(source);
        let kill = put(&mut g, p, "Murder", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: kill });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(g.object_view(source).controller, p);
        assert_eq!(g.object_view(source).owner, p);
        assert_ne!(g.object_ref(source), old);
        assert!(!g.is_creature(source));
        assert_eq!(g.current_kinds(source) & 33, 32);
        assert_eq!(g.object_view(source).name, "Enduring Curiosity");
        assert!(g.keywords(source).contains(Keywords::FLASH));
        let before = g.snapshot();
        let mut child = g.clone();
        child.restore(&before).unwrap();
        assert_eq!(child.snapshot(), before);
        g.move_object(source, Zone::Exile);
        let entry = g.entry_move(source, Zone::Battlefield, false);
        g.stage_entries(vec![entry], None);
        g.advance_entries();
        settle(&mut g);
        assert!(g.is_creature(source));
    }
}

#[test]
fn curiosity_exile_replacement_prevents_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
        let source = put(&mut g, p, "Enduring Curiosity", Zone::Battlefield);
        let kill = put(&mut g, p, "Murder", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: kill });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
    }
}

#[test]
fn curiosity_draws_per_controlled_creature_combat_hit_even_after_returning() {
    for p in [Player::P0, Player::P1] {
        for returned in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Enduring Curiosity", Zone::Battlefield);
            if returned {
                let kill = put(&mut g, p, "Murder", Zone::Hand);
                checked(&mut g, Action::AnnounceCast { card: kill });
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(source),
                    },
                );
                pay(&mut g);
                settle(&mut g);
                assert!(!g.is_creature(source));
            }
            let first = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let second = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let draws = g.state.players[p.index()].cards_drawn_this_turn;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: first });
            checked(&mut g, Action::Attack { card: second });
            checked(&mut g, Action::FinishAttackers);
            for _ in 0..60 {
                if g.state.players[p.index()].cards_drawn_this_turn == draws + 2 {
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
                    .expect("combat continuation");
                checked(&mut g, action);
            }
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws + 2);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        }
    }
}
