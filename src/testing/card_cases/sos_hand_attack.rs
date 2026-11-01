//! Focused printed outcomes; shared engine invariants live in core tests.
//! Paid partial targets, private permanent placement, and exact temporary attack grants.
use super::big_helpers::{cast as paid, checked, settle as finish};
use super::*;

#[test]
fn sos_hand_attack_root_freezes_recipients_stacks_triggers_and_expires() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let c = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        for _ in 0..2 {
            let spell = put(&mut g, p, "Root Manipulation", Zone::Hand);
            paid(&mut g, spell, None, &[]);
            finish(&mut g);
        }
        for id in [a, b] {
            assert_eq!(stats(&g, id), (6, 6));
            assert!(g.keywords(id).contains(Keywords::MENACE));
        }
        assert_eq!(stats(&g, c), (2, 2));
        assert!(!g.keywords(c).contains(Keywords::MENACE));
        let newcomer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, newcomer), (2, 2));
        g.move_object(b, Zone::Hand);
        g.move_object(b, Zone::Battlefield);
        assert_eq!(stats(&g, b), (2, 2));
        assert!(!g.keywords(b).contains(Keywords::MENACE));
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: a });
        checked(&mut g, Action::Attack { card: newcomer });
        checked(&mut g, Action::FinishAttackers);
        assert_eq!(
            g.state
                .stack
                .iter()
                .filter(|s| s.source.unwrap().card == Card::from_name("Root Manipulation").unwrap())
                .count()
                + g.state.ordering_triggers.len(),
            2
        );
        g.move_object(a, Zone::Exile);
        finish(&mut g);
        assert_eq!(
            g.state.players[p.index()].life,
            22,
            "each granted attack ability survives recipient departure"
        );
        g.finish_cleanup();
        assert!(g.observe(p).temporary_abilities.is_empty());
        assert_eq!(stats(&g, b), (2, 2));
    }
}
