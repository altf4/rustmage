//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("inherited combat decision budget: {:?}", g.state.pending);
}
fn drive(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn bind(g: &mut Game, id: CardId) {
    until(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
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
fn attack(g: &mut Game, ids: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &id in ids {
        g.state.objects[id as usize].summoning_sick = false;
        act(g, Action::Attack { card: id });
    }
    act(g, Action::FinishAttackers);
}
fn blockers(g: &mut Game) {
    until(g, |g| g.state.pending == Pending::Blockers);
}
fn end_combat(g: &mut Game) {
    act(g, Action::FinishBlockers);
    for _ in 0..32 {
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
    panic!("inherited combat damage budget");
}
#[test]
fn inherited_combat_bat_paid_morbid_any_controller_and_real_flying_deathtouch() {
    {
        let p = Player::P0;
        for died_controller in [None, Some(p), Some(p.other())] {
            let mut g = scenario(p);
            if let Some(controller) = died_controller {
                let fodder = put(&mut g, controller, "Bear Cub", Zone::Battlefield);
                let shock = put(&mut g, p, "Shock", Zone::Hand);
                g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
                cast(&mut g, shock, Some(Target::Card(fodder)));
                drive(&mut g);
                assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
                assert_eq!(g.observe(p).creatures_died_this_turn, 1);
            } else {
                let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
                g.move_object(artifact, Zone::Graveyard);
                g.give_priority();
                let bat = put(&mut g, p, "Dreaded Bat-Cloud", Zone::Hand);
                g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
                reject(&mut g, Action::AnnounceCast { card: bat });
            }
            let bat = paid(
                &mut g,
                p,
                "Dreaded Bat-Cloud",
                [0, 0, 1, 0, 0, if died_controller.is_some() { 1 } else { 4 }],
            );
            drive(&mut g);
            assert_eq!(stats(&g, bat), (4, 2));
            assert!(
                g.keywords(bat)
                    .contains(Keywords::FLYING.union(Keywords::DEATHTOUCH))
            );
            let ground = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spider = put(&mut g, p.other(), "Old Fat Spider", Zone::Battlefield);
            attack(&mut g, &[bat]);
            blockers(&mut g);
            reject(
                &mut g,
                Action::Block {
                    blocker: ground,
                    attacker: bat,
                },
            );
            act(
                &mut g,
                Action::Block {
                    blocker: spider,
                    attacker: bat,
                },
            );
            end_combat(&mut g);
            assert_eq!(g.object_view(spider).zone, Zone::Graveyard);
            assert_eq!(g.object_view(bat).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn inherited_combat_hunter_both_hybrids_any_creature_counter_and_token_block_exclusion() {
    {
        let p = Player::P0;
        for black in [false, true] {
            let mut g = scenario(p);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let hunter = paid(
                &mut g,
                p,
                "Duskwatch Hunter",
                if black {
                    [0, 0, 1, 0, 0, 2]
                } else {
                    [0, 0, 0, 0, 1, 2]
                },
            );
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            let land = put(&mut g, p, "Forest", Zone::Battlefield);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            bind(&mut g, if black { enemy } else { hunter });
            drive(&mut g);
            assert_eq!(
                g.counter_count(if black { enemy } else { hunter }, CounterKind::PlusOne),
                1
            );
            assert_eq!(stats(&g, hunter), if black { (3, 1) } else { (4, 2) });
            let token = put(&mut g, p.other(), "@token/hob-bear", Zone::Battlefield);
            attack(&mut g, &[hunter]);
            blockers(&mut g);
            reject(
                &mut g,
                Action::Block {
                    blocker: token,
                    attacker: hunter,
                },
            );
            act(
                &mut g,
                Action::Block {
                    blocker: enemy,
                    attacker: hunter,
                },
            );
        }
    }
}

#[test]
fn inherited_combat_rabbit_own_combat_ferocious_rechecked_and_chief_trigger_only_once_per_attack() {
    {
        let p = Player::P0;
        for row in 0..3 {
            let mut g = scenario(p);
            let rabbit = paid(&mut g, p, "Nasty Little Rabbit", [0, 0, 0, 0, 1, 0]);
            drive(&mut g);
            assert_eq!(stats(&g, rabbit), (1, 2));
            let strong = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            if row > 0 {
                g.add_counter(strong, CounterKind::PlusOne, 1);
            }
            put(&mut g, p.other(), "Large Bear", Zone::Battlefield);
            g.state.active = p.other();
            g.begin_step(Step::BeginCombat);
            g.give_priority();
            g.prepare_triggers();
            drive(&mut g);
            assert_eq!(g.counter_count(rabbit, CounterKind::PlusOne), 0);
            g.state.active = p;
            g.begin_step(Step::BeginCombat);
            g.give_priority();
            g.prepare_triggers();
            if row == 2 {
                until(&mut g, |g| !g.state.stack.is_empty());
                g.move_object(strong, Zone::Graveyard);
                g.give_priority();
            }
            drive(&mut g);
            assert_eq!(
                g.counter_count(rabbit, CounterKind::PlusOne),
                if row == 1 { 1 } else { 0 }
            );
        }
        for qualify in [false, true] {
            let mut g = scenario(p);
            let chief = paid(&mut g, p, "The Chief Warg", [0, 0, 1, 0, 1, 2]);
            drive(&mut g);
            assert_eq!(stats(&g, chief), (3, 3));
            assert!(g.keywords(chief).contains(Keywords::MENACE));
            let strong = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            if qualify {
                g.add_counter(strong, CounterKind::PlusOne, 1);
            }
            let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            attack(&mut g, &[a, b]);
            if qualify {
                until(&mut g, |g| !g.state.stack.is_empty());
                g.move_object(strong, Zone::Graveyard);
                g.give_priority();
            }
            blockers(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                if qualify { 1 } else { 0 }
            );
            assert_eq!(
                g.state.players[p.index()].life,
                if qualify { 19 } else { 20 }
            );
        }
        // Menace rejects completing a single-blocker group, then accepts two.
        let mut g = scenario(p);
        let chief = paid(&mut g, p, "The Chief Warg", [0, 0, 1, 0, 1, 2]);
        drive(&mut g);
        let a = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        attack(&mut g, &[chief]);
        blockers(&mut g);
        act(
            &mut g,
            Action::Block {
                blocker: a,
                attacker: chief,
            },
        );
        reject(&mut g, Action::FinishBlockers);
        act(
            &mut g,
            Action::Block {
                blocker: b,
                attacker: chief,
            },
        );
        end_combat(&mut g);
    }
}
#[test]
fn inherited_combat_nori_both_hybrids_attacking_target_real_first_strike_and_expiry() {
    {
        let p = Player::P0;
        for white in [false, true] {
            let mut g = scenario(p);
            let nori = paid(
                &mut g,
                p,
                "Nori, Teller of Tales",
                if white {
                    [1, 0, 0, 0, 0, 1]
                } else {
                    [0, 0, 0, 1, 0, 1]
                },
            );
            drive(&mut g);
            assert_eq!(stats(&g, nori), (2, 2));
            let friend = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let idle = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            attack(&mut g, &[nori, friend]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(idle),
                },
            );
            bind(&mut g, if white { friend } else { nori });
            blockers(&mut g);
            let recipient = if white { friend } else { nori };
            assert!(g.keywords(recipient).contains(Keywords::FIRST_STRIKE));
            act(
                &mut g,
                Action::Block {
                    blocker,
                    attacker: recipient,
                },
            );
            end_combat(&mut g);
            assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
            assert_eq!(g.object_view(recipient).zone, Zone::Battlefield);
            assert_eq!(g.state.objects[recipient as usize].damage, 0);
            g.finish_cleanup();
            assert!(!g.keywords(recipient).contains(Keywords::FIRST_STRIKE));
        }
    }
}
#[test]
fn inherited_combat_spider_paid_reach_power_boundary_and_opponent_spell_or_ability_draw() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let spider = paid(&mut g, p, "Old Fat Spider", [0, 0, 0, 0, 2, 4]);
        drive(&mut g);
        assert_eq!(stats(&g, spider), (6, 7));
        assert!(g.keywords(spider).contains(Keywords::REACH));
        let small = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let large = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        assert!(!g.can_block_pair(small, spider));
        assert!(g.can_block_pair(large, spider));
        let flyer = put(&mut g, p.other(), "Ravenhill Flock", Zone::Battlefield);
        g.state.active = p.other();
        assert!(g.can_block_pair(spider, flyer));
        g.state.active = p;
        let own = put(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        cast(&mut g, own, Some(Target::Card(spider)));
        drive(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        g.state.priority = p.other();
        let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 0];
        cast(&mut g, shock, Some(Target::Card(spider)));
        drive(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        let harper = put(&mut g, p.other(), "Elvenking's Harper", Zone::Battlefield);
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 4];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: harper,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        target_action(&mut g, spider);
        pay(&mut g);
        g.move_object(spider, Zone::Graveyard);
        g.give_priority();
        drive(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    }
}
fn target_action(g: &mut Game, id: CardId) {
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
#[test]
fn inherited_combat_eagles_flash_sum_current_owned_flying_power_and_colored_floor() {
    {
        let p = Player::P0;
        for sum in [0, 2, 7, 10] {
            let mut g = scenario(p);
            put(&mut g, p.other(), "Ravenhill Flock", Zone::Battlefield);
            put(&mut g, p, "Hill Giant", Zone::Battlefield);
            if sum > 0 {
                let a = put(&mut g, p, "Ravenhill Flock", Zone::Battlefield);
                let b = put(&mut g, p, "Ravenhill Flock", Zone::Battlefield);
                // Both are actual flying creatures; current power, including a
                // negative contribution, is summed before generic reduction.
                g.state.objects[a as usize].growth.power = if sum == 2 { -2 } else { 2 };
                g.state.objects[b as usize].growth.power = if sum == 2 { 2 } else { sum - 4 };
            }
            g.state.active = p.other();
            g.state.step = Step::End;
            g.state.priority = p;
            let eagle = put(&mut g, p, "The Lord of the Eagles", Zone::Hand);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 20];
            reject(&mut g, Action::AnnounceCast { card: eagle });
            g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 7u32.saturating_sub(sum as u32)];
            cast(&mut g, eagle, None);
            drive(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(stats(&g, eagle), (8, 8));
            assert!(
                g.keywords(eagle)
                    .contains(Keywords::FLASH.union(Keywords::FLYING))
            );
        }
    }
}
