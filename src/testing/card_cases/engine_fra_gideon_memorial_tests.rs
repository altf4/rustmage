//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

#[test]
fn memorial_discard_ability_targets_only_attacking_or_blocking_creatures() {
    {
        let p = Player::P0;
        let mut g = scenario(p.other());
        let attacker = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let idle = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let memorial = put(&mut g, p, "Gideon's Memorial", Zone::Hand);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        g.state.priority = p;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: memorial,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(idle),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(attacker),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(memorial).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(attacker).zone, Zone::Graveyard);
    }
}
