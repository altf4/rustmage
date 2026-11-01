//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;
fn drive(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::PutTrigger { .. } | Action::Pass));
    }
    panic!("creature resolution budget");
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None);
    drive(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn start(g: &mut Game, id: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
fn activate(g: &mut Game, id: CardId, target: Option<CardId>) {
    start(g, id);
    if let Some(id) = target {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(g, Action::FinishTargets);
        }
    }
    pay(g);
}
fn combat(g: &mut Game, attacker: CardId, blocker: Option<CardId>) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: attacker });
    act(g, Action::FinishAttackers);
    for _ in 0..12 {
        if g.state.pending == Pending::Blockers {
            break;
        }
        act(g, Action::Pass);
    }
    assert_eq!(g.state.pending, Pending::Blockers);
    if let Some(blocker) = blocker {
        act(g, Action::Block { blocker, attacker });
    }
    act(g, Action::FinishBlockers);
    for _ in 0..24 {
        if g.state.step == Step::EndCombat {
            return;
        }
        if let Some(range) = g.damage_assignment_range() {
            act(
                g,
                Action::AssignDamage {
                    target: range.target,
                    amount: range.minimum,
                },
            );
        } else {
            act(g, Action::Pass);
        }
    }
    panic!("combat budget");
}

#[test]
fn hob_inherited_creatures_ordinary_paid_body_and_large_all_hybrids_haste_trample_reach() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let ordinary = paid(&mut g, p, "Ordinary Bear", [0, 0, 0, 0, 1, 3]);
        assert_eq!(stats(&g, ordinary), (4, 5));
        assert!(g.state.objects[ordinary as usize].card.has_subtype("Bear"));
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::Attack { card: ordinary });
        for mana in [[0, 0, 2, 0, 0, 3], [0, 0, 1, 0, 1, 3], [0, 0, 0, 0, 2, 3]] {
            let mut g = scenario(p);
            let bear = paid(&mut g, p, "Large Bear", mana);
            assert_eq!(stats(&g, bear), (5, 5));
            assert!(
                g.keywords(bear).contains(
                    Keywords::REACH
                        .union(Keywords::TRAMPLE)
                        .union(Keywords::HASTE)
                )
            );
            assert!(g.object_view(bear).summoning_sick);
            let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            combat(&mut g, bear, Some(blocker));
            assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.other().index()].life, 17);
            assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
            let mut reach = scenario(p);
            let bear = paid(&mut reach, p, "Large Bear", mana);
            let flyer = put(&mut reach, p.other(), "Ravenhill Flock", Zone::Battlefield);
            reach.state.active = p.other();
            assert!(reach.can_block_pair(bear, flyer));
            reach.state.objects[bear as usize].tapped = true;
            assert!(!reach.can_block_pair(bear, flyer));
        }
    }
}
#[test]
fn hob_inherited_creatures_gigantic_counter_resistance_hexproof_and_hasty_damage() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Gigantic Big Bear", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 5];
        cast(&mut g, bear, None);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.priority = p.other();
        let counter = put(&mut g, p.other(), "Essence Scatter", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, counter, Some(Target::Spell(bear)));
        drive(&mut g);
        assert_eq!(g.object_view(counter).zone, Zone::Graveyard);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(stats(&g, bear), (10, 7));
        assert!(
            g.keywords(bear)
                .contains(Keywords::HEXPROOF.union(Keywords::HASTE))
        );
        g.state.priority = p.other();
        let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 0];
        act(&mut g, Action::AnnounceCast { card: shock });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        // Inspect own targeting in an independent branch; the opponent's failed
        // announcement never makes our permanent targetable to them.
        let mut own = scenario(p);
        let own_bear = paid(&mut own, p, "Gigantic Big Bear", [0, 0, 0, 0, 2, 5]);
        let growth = put(&mut own, p, "Giant Growth", Zone::Hand);
        own.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        cast(&mut own, growth, Some(Target::Card(own_bear)));
        drive(&mut own);
        assert_eq!(stats(&own, own_bear), (13, 10));
        own.finish_cleanup();
        assert_eq!(stats(&own, own_bear), (10, 7));
        let mut hasty = scenario(p);
        let attacker = paid(&mut hasty, p, "Gigantic Big Bear", [0, 0, 0, 0, 2, 5]);
        assert!(hasty.object_view(attacker).summoning_sick);
        combat(&mut hasty, attacker, None);
        assert_eq!(hasty.state.players[p.other().index()].life, 10);
    }
}

#[test]
fn hob_inherited_creatures_provisioner_fixed_current_group_and_guardian_permanent_counters() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = paid(&mut g, p, "Dwarven Provisioner", [1, 0, 0, 0, 0, 1]);
        assert_eq!(stats(&g, source), (2, 2));
        let friend = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 3];
        activate(&mut g, source, None);
        drive(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(stats(&g, source), (3, 3));
        assert_eq!(stats(&g, friend), (3, 3));
        assert_eq!(stats(&g, enemy), (2, 2));
        assert_eq!(g.state.objects[land as usize].growth, Stats::default());
        let late = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, late), (2, 2));
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 3];
        activate(&mut g, source, None);
        g.move_object(source, Zone::Graveyard);
        g.give_priority();
        drive(&mut g);
        assert_eq!(stats(&g, friend), (4, 4));
        assert_eq!(stats(&g, late), (3, 3));
        g.finish_cleanup();
        assert_eq!(stats(&g, friend), (2, 2));
        assert_eq!(stats(&g, late), (2, 2));
        let mut g = scenario(p);
        let guardian = paid(&mut g, p, "Guardian of the Halls", [0, 0, 0, 0, 1, 1]);
        assert_eq!(stats(&g, guardian), (2, 2));

        for n in 1..=2 {
            g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 5];
            activate(&mut g, guardian, None);
            drive(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.counter_count(guardian, CounterKind::PlusOne), 3 * n);
            assert_eq!(
                stats(&g, guardian),
                (2 + i128::from(3 * n), 2 + i128::from(3 * n))
            );
        }
        g.finish_cleanup();
        assert_eq!(stats(&g, guardian), (8, 8));
        // Cleanup advanced to the opponent's upkeep. They pass priority before
        // the Guardian's controller activates this instant-speed ability.
        act(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(p));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 5];
        activate(&mut g, guardian, None);
        g.move_object(guardian, Zone::Hand);
        g.move_object(guardian, Zone::Battlefield);
        g.give_priority();
        drive(&mut g);
        assert_eq!(g.counter_count(guardian, CounterKind::PlusOne), 0);
    }
}
#[test]
fn hob_inherited_creatures_harper_any_controller_legality_expiry_and_exact_recipient() {
    {
        let p = Player::P0;
        for owner in [p, p.other()] {
            let mut g = scenario(p);
            let source = paid(&mut g, p, "Elvenking's Harper", [0, 1, 0, 0, 0, 1]);
            assert_eq!(stats(&g, source), (2, 2));
            let recipient = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
            let blocker = put(&mut g, owner.other(), "Bear Cub", Zone::Battlefield);
            let land = put(&mut g, p, "Forest", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 4];
            start(&mut g, source);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(recipient),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let mut stale = g.clone();
            stale.move_object(recipient, Zone::Hand);
            stale.move_object(recipient, Zone::Battlefield);
            stale.give_priority();
            drive(&mut stale);
            stale.state.active = owner;
            assert!(stale.can_block_pair(blocker, recipient));
            g.move_object(source, Zone::Graveyard);
            g.give_priority();
            drive(&mut g);
            g.state.active = owner;
            assert!(!g.can_block_pair(blocker, recipient));
            g.finish_cleanup();
            g.state.active = owner;
            assert!(g.can_block_pair(blocker, recipient));
        }
    }
}
