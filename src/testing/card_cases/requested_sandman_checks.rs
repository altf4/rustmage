//! Focused printed outcomes; shared engine invariants live in core tests.
//! Sandman's blocking threshold in an actual combat.
use super::*;

#[test]
fn sandman_rejects_two_power_blocker_and_allows_three_power() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        for _ in 0..3 {
            put(&mut g, p, "Forest", Zone::Battlefield);
        }
        let sandman = put(&mut g, p, "Sandman, Shifting Scoundrel", Zone::Battlefield);
        let small = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let large = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[sandman as usize].summoning_sick = false;
        assert_eq!(g.stats(sandman).power, 3);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        big_helpers::checked(&mut g, Action::Attack { card: sandman });
        big_helpers::checked(&mut g, Action::FinishAttackers);
        for _ in 0..40 {
            if matches!(g.state.pending, Pending::Blockers) {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::Blockers));
        reject(
            &mut g,
            Action::Block {
                blocker: small,
                attacker: sandman,
            },
        );
        big_helpers::checked(
            &mut g,
            Action::Block {
                blocker: large,
                attacker: sandman,
            },
        );
        big_helpers::checked(&mut g, Action::FinishBlockers);
    }
}
