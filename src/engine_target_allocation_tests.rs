//! Actual loyalty declarations, fixed target shares and paid exile permissions.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;

fn chandra(g: &mut Game, p: Player) -> CardId {
    let id = super::tests::spawn(
        g,
        p,
        Card::from_name("Chandra, Flameshaper").unwrap(),
        Zone::Hand,
    );
    g.state.players[p.index()].mana = [0, 0, 0, 7, 0, 0];
    cast(g, id, None, &[]);
    settle(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.counter_count(id, CounterKind::Loyalty), 6);
    id
}
fn announce(g: &mut Game, id: CardId, ability: u16, targets: &[Target]) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
}
fn pay(g: &mut Game) {
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(g, action);
    }
    panic!("unpaid ability");
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
#[test]
fn chandra_zero_target_minus_four_is_legal_and_locks_every_loyalty_ability() {
    assert_eq!(std::mem::size_of::<Object>(), 144);
    assert_eq!(std::mem::size_of::<Pending>(), 40);
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = chandra(&mut g, p);
        announce(&mut g, id, 2, &[]);
        pay(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Loyalty), 2);
        assert!(g.state.stack.last().unwrap().targets.is_empty());
        assert!(g.state.stack.last().unwrap().target_allocations.is_none());
        settle(&mut g);
        for ability in 0..3 {
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability,
                    counter_payment: Counters::default(),
                },
            );
        }
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn declared_damage_requires_positive_total_eight_and_never_reallocates_an_illegal_share() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = chandra(&mut g, p);
        let a = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        // Chandra herself is a legal planeswalker target; her cost leaves two loyalty.
        announce(
            &mut g,
            id,
            2,
            &[Target::Card(a), Target::Card(b), Target::Card(id)],
        );
        assert_eq!(g.observe(p).target_amount.unwrap().maximum, 6);
        assert!(g.observe(p.other()).actions.is_empty());
        reject(&mut g, Action::AllocateTargetAmount { amount: 0 });
        reject(&mut g, Action::AllocateTargetAmount { amount: 7 });
        checked(&mut g, Action::AllocateTargetAmount { amount: 3 });
        checked(&mut g, Action::AllocateTargetAmount { amount: 4 });
        assert_eq!(g.observe(p).target_amount.unwrap().minimum, 1);
        reject(&mut g, Action::AllocateTargetAmount { amount: 2 });
        checked(&mut g, Action::AllocateTargetAmount { amount: 1 });
        pay(&mut g);
        let item = g.state.stack.last().unwrap();
        assert_eq!(
            item.target_allocations
                .as_ref()
                .unwrap()
                .iter()
                .map(|r| r.amount)
                .sum::<u16>(),
            8
        );
        g.move_object(a, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.state.objects[a as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[b as usize].zone, Zone::Graveyard);
        assert_eq!(g.counter_count(id, CounterKind::Loyalty), 1);
    }
}
#[test]
fn single_divided_share_survives_exact_bolt_bend_retarget_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = chandra(&mut g, p);
        let a = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let b = named(&mut g, p, "Serra Angel", Zone::Battlefield);
        announce(&mut g, id, 2, &[Target::Card(a)]);
        assert_eq!(g.observe(p).target_amount.unwrap().minimum, 8);
        checked(&mut g, Action::AllocateTargetAmount { amount: 8 });
        pay(&mut g);
        let ability = g.state.stack.last().unwrap().id;
        checked(&mut g, Action::Pass);
        let bend = named(&mut g, p.other(), "Bolt Bend", Zone::Hand);
        cast(&mut g, bend, None, &[Target::Stack(ability)]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert!(g.observe(p.other()).retarget.is_some());
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(b),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        let item = g.state.stack.iter().find(|s| s.id == ability).unwrap();
        assert_eq!(
            item.target_allocations.as_ref().unwrap()[0].target.target,
            Target::Card(b)
        );
        assert_eq!(item.target_allocations.as_ref().unwrap()[0].amount, 8);
        g.move_object(id, Zone::Graveyard);
        settle(&mut g);
        assert!(g.on_battlefield(a));
        assert_eq!(g.state.objects[b as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn chandra_plus_two_uses_the_stack_and_grants_only_the_chosen_exact_paid_play_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = chandra(&mut g, p);
        let land = named(&mut g, p, "Mountain", Zone::Library);
        let creature = named(&mut g, p, "Bear Cub", Zone::Library);
        let instant = named(&mut g, p, "Think Twice", Zone::Library);
        announce(&mut g, id, 0, &[]);
        pay(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Loyalty), 8);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ActivateMana { card, .. } if *card == id))
        );
        g.move_object(id, Zone::Graveyard);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 3, 0, 0]);
        for card in [land, creature, instant] {
            assert_eq!(g.state.objects[card as usize].zone, Zone::Exile);
        }
        assert!(g.observe(p.other()).actions.is_empty());
        let object = g.object_ref(land);
        checked(&mut g, Action::SelectMilled { object });
        if g.legal_actions().contains(&Action::FinishMilledSelection) {
            checked(&mut g, Action::FinishMilledSelection);
        }
        settle(&mut g);
        assert!(g.can_play_exiled(p, land));
        assert!(!g.can_play_exiled(p, creature));
        assert!(!g.can_play_exiled(p, instant));
        assert!(!g.can_play_exiled(p.other(), land));
        checked(&mut g, Action::PlayLand { card: land });
        assert!(g.on_battlefield(land));
        assert!(!g.can_play_exiled(p, land));
    }
}
#[test]
fn chandra_plus_one_copies_controlled_creature_with_abilities_after_chandra_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = chandra(&mut g, p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let other = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(other),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        pay(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Loyalty), 7);
        g.move_object(id, Zone::Graveyard);
        settle(&mut g);
        let token = g.state.copy_recipes.as_ref().unwrap()[0].object.card;
        assert_eq!(
            g.stats(token),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(g.keywords(token).contains(Keywords::HASTE));
        g.begin_step(Step::End);
        settle(&mut g);
        assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
    }
}

#[test]
fn smoke_policy_values_an_own_creature_copy_over_an_untargeted_value_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = chandra(&mut g, p);
        let creature = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let actions: Vec<_> = g.legal_actions().into_iter().filter(|a| {
            matches!(a, Action::Activate { card, ability: 0, target: None, .. } if *card == id)
                || matches!(a, Action::Activate { card, ability: 1, target: Some(Target::Card(target)), .. } if *card == id && *target == creature)
        }).collect();
        assert_eq!(actions.len(), 2);
        assert!(matches!(
            actions[crate::policy::smoke_index(&g, &actions)],
            Action::Activate { ability: 1, target: Some(Target::Card(target)), .. } if target == creature
        ));
    }
}
