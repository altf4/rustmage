//! Focused printed outcomes; shared engine invariants live in core tests.
use super::big_helpers::{checked, settle};
use super::*;

fn activate(g: &mut Game, monument: CardId, ability: u16, players: &[Player]) {
    checked(
        g,
        Action::AnnounceActivate {
            card: monument,
            ability,
            counter_payment: Counters::default(),
        },
    );
    for &player in players {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Player(player),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
    pay(g);
    settle(g);
}

#[test]
fn suppression_batch_riverchurn_mills_selected_players_by_separate_graveyard_counts_and_exhausts_once()
 {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let monument = put(&mut g, p, "Riverchurn Monument", Zone::Battlefield);
        put(&mut g, p, "Forest", Zone::Graveyard);
        for _ in 0..3 {
            put(&mut g, p.other(), "Forest", Zone::Graveyard);
        }
        activate(&mut g, monument, 0, &[p, p.other()]);
        assert_eq!(g.state.players[p.index()].library.len(), 6);
        assert_eq!(g.state.players[p.other().index()].library.len(), 6);
        g.state.objects[monument as usize].untap();
        activate(&mut g, monument, 1, &[p, p.other()]);
        assert_eq!(g.state.players[p.index()].library.len(), 3);
        assert_eq!(g.state.players[p.other().index()].library.len(), 1);
        g.state.objects[monument as usize].untap();
        assert!(!g.legal_actions().contains(&Action::AnnounceActivate {
            card: monument,
            ability: 1,
            counter_payment: Counters::default()
        }));
        let libraries = [
            g.state.players[0].library.len(),
            g.state.players[1].library.len(),
        ];
        activate(&mut g, monument, 0, &[]);
        assert_eq!(
            [
                g.state.players[0].library.len(),
                g.state.players[1].library.len()
            ],
            libraries
        );
        // A new battlefield incarnation has a fresh exhaust use.
        g.move_object(monument, Zone::Hand);
        g.move_object(monument, Zone::Battlefield);
        assert!(g.legal_actions().contains(&Action::AnnounceActivate {
            card: monument,
            ability: 1,
            counter_payment: Counters::default()
        }));
    }
}
