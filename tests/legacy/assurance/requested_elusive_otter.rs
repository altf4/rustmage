//! Elusive Otter and Grove's Bounty paid rules interactions.
use super::big_helpers::{checked, settle, setup};
use super::*;

#[test]
fn elusive_otter_relative_power_restriction_and_prowess() {
    for p in [Player::P0, Player::P1] {
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
    for p in [Player::P0, Player::P1] {
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

#[test]
fn grove_bounty_zero_x_requires_no_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spell = put(&mut g, p, "Elusive Otter // Grove's Bounty", Zone::Hand);
        checked(&mut g, Action::AnnounceAdventure { card: spell });
        reject(&mut g, Action::ChooseX { value: 1 });
        checked(&mut g, Action::ChooseX { value: 0 });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
    }
}

#[test]
fn grove_bounty_preserves_division_when_a_target_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 3];
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Elusive Otter // Grove's Bounty", Zone::Hand);
        checked(&mut g, Action::AnnounceAdventure { card: spell });
        checked(&mut g, Action::ChooseX { value: 3 });
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
        checked(&mut g, Action::AllocateTargetAmount { amount: 1 });
        checked(&mut g, Action::AllocateTargetAmount { amount: 2 });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        g.move_object(b, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 1);
        assert_eq!(g.object_view(b).zone, Zone::Hand);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
    }
}

#[test]
fn grove_bounty_large_division_accepts_full_u32_values() {
    let p = Player::P0;
    let mut g = setup(p);
    let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut g, p, "Elusive Otter // Grove's Bounty", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 70_000];
    checked(&mut g, Action::AnnounceAdventure { card: spell });
    checked(&mut g, Action::ChooseX { value: 70_000 });
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
    let action = Action::AllocateTargetAmount { amount: 34_567 };
    assert!(
        !g.legal_actions().contains(&action),
        "large domains expose bounded representatives"
    );
    let snapshot = g.snapshot();
    assert_eq!(g.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(g.snapshot(), snapshot);
    g.apply(p, action).unwrap();
    checked(&mut g, Action::AllocateTargetAmount { amount: 35_433 });
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(&mut g, pay);
    settle(&mut g);
    assert_eq!(g.counter_count(a, CounterKind::PlusOne), 34_567);
    assert_eq!(g.counter_count(b, CounterKind::PlusOne), 35_433);
}

#[test]
fn elusive_otter_front_identity_and_adventure_face_are_distinct() {
    let card = Card::from_name("Elusive Otter // Grove's Bounty").unwrap();
    assert_eq!(Card::from_name("Elusive Otter"), Some(card));
    assert_eq!(card.name(), "Elusive Otter");
    assert_eq!(card.info().mana_cost, "{U}");
    assert_eq!(card.fixed_mana_value(), 1);
    assert_eq!(card.adventure_spell(), Card::from_name("Grove's Bounty"));
    assert_eq!(card.adventure_spell().unwrap().info().mana_cost, "{X}{G}");
}

#[test]
fn elusive_otter_block_restriction_tracks_copied_definition() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let copied = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let otter = put(
            &mut g,
            p,
            "Elusive Otter // Grove's Bounty",
            Zone::Battlefield,
        );
        let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[blocker as usize].growth.power = -2;
        assert!(g.can_block_pair(blocker, copied));
        assert!(!g.can_block_pair(blocker, otter));
        let recipe = CopyRecipe::new(Card::from_name("Elusive Otter // Grove's Bounty").unwrap());
        g.apply_temporary_copy(g.object_ref(copied), recipe, CopyDeadline::EndOfTurn);
        assert!(!g.can_block_pair(blocker, copied));
        let bear = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        g.apply_temporary_copy(g.object_ref(otter), bear, CopyDeadline::EndOfTurn);
        assert!(g.can_block_pair(blocker, otter));
        super::contracts::branch_contract(g.clone());
    }
}

#[test]
fn elusive_otter_loses_restriction_with_abilities_and_regains_it_when_restored() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let otter = put(
            &mut g,
            p,
            "Elusive Otter // Grove's Bounty",
            Zone::Battlefield,
        );
        let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[blocker as usize].growth.power = -2;
        assert!(!g.can_block_pair(blocker, otter));
        let aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        big_helpers::cast(&mut g, aura, None, &[Target::Card(otter)]);
        settle(&mut g);
        assert!(!g.printed_abilities_available(otter));
        assert!(g.can_block_pair(blocker, otter));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.printed_abilities_available(otter));
        assert!(!g.can_block_pair(blocker, otter));
    }
}
#[test]
fn grove_bounty_all_targets_illegal_and_allocation_branch_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 3];
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Elusive Otter // Grove's Bounty", Zone::Hand);
        checked(&mut g, Action::AnnounceAdventure { card: spell });
        checked(&mut g, Action::ChooseX { value: 3 });
        for card in [a, b] {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        checked(&mut g, Action::FinishTargets);
        super::contracts::branch_contract(g.clone());
        checked(&mut g, Action::AllocateTargetAmount { amount: 1 });
        super::contracts::branch_contract(g.clone());
        checked(&mut g, Action::AllocateTargetAmount { amount: 2 });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        g.move_object(a, Zone::Hand);
        g.move_object(b, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
    }
}

#[test]
fn grove_bounty_loses_target_when_controller_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 3];
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Elusive Otter // Grove's Bounty", Zone::Hand);
        checked(&mut g, Action::AnnounceAdventure { card: spell });
        checked(&mut g, Action::ChooseX { value: 3 });
        for card in [a, b] {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        checked(&mut g, Action::FinishTargets);
        checked(&mut g, Action::AllocateTargetAmount { amount: 1 });
        checked(&mut g, Action::AllocateTargetAmount { amount: 2 });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        g.state.objects[b as usize].controller = p.other();
        settle(&mut g);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
    }
}
