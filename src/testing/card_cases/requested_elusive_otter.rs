//! Focused printed outcomes; shared engine invariants live in core tests.
//! Elusive Otter and Grove's Bounty paid rules interactions.
use super::big_helpers::{checked, settle, setup};
use super::*;

#[test]
fn elusive_otter_relative_power_restriction_and_prowess() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let otter = put(
            &mut g,
            p,
            "Elusive Otter // Grove's Bounty",
            Zone::Battlefield,
        );
        let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert!(g.can_block_pair(blocker, otter));
        g.state.objects[blocker as usize].growth.power = -2;
        assert!(!g.can_block_pair(blocker, otter));
        g.state.objects[blocker as usize].growth.power = -1;
        assert!(g.can_block_pair(blocker, otter));
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        big_helpers::cast(&mut g, spell, None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.stats(otter).power, 2);
        assert!(!g.can_block_pair(blocker, otter));
        g.state.objects[blocker as usize].growth.power = 0;
        assert!(g.can_block_pair(blocker, otter));
        g.finish_cleanup();
        assert_eq!(g.stats(otter).power, 1);
    }
}

#[test]
fn grove_bounty_divides_x_across_controlled_creatures_and_adventures() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 3];
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Elusive Otter // Grove's Bounty", Zone::Hand);
        checked(&mut g, Action::AnnounceAdventure { card: spell });
        checked(&mut g, Action::ChooseX { value: 3 });
        reject(&mut g, Action::FinishTargets);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(b),
            },
        );
        checked(&mut g, Action::FinishTargets);
        reject(&mut g, Action::AllocateTargetAmount { amount: 0 });
        checked(&mut g, Action::AllocateTargetAmount { amount: 1 });
        checked(&mut g, Action::AllocateTargetAmount { amount: 2 });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 2);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        g.state.players[p.index()].mana[1] = 1;
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceExile { card: spell })
        );
    }
}
