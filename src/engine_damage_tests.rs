use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
use std::collections::BTreeSet;

fn permanent(game: &mut Game, owner: Player, name: &str) -> CardId {
    spawn(
        game,
        owner,
        Card::from_name(name).unwrap(),
        Zone::Battlefield,
    )
}

fn combat(game: &mut Game, attacker: CardId, blockers: &[CardId]) {
    game.state.attackers.push(Attacker {
        card: attacker,
        blocked: !blockers.is_empty(),
    });
    for &blocker in blockers {
        game.state.blocks.push((blocker, attacker));
        game.state.blocking.push(blocker);
    }
    game.state.step = Step::Blockers;
    game.give_priority();
}

#[test]
fn first_strike_kills_before_retaliation_and_restores_between_steps() {
    let mut game = scenario();
    let stoat = permanent(&mut game, Player::P0, "Brightblade Stoat");
    let bears = permanent(&mut game, Player::P1, "Grizzly Bears");
    combat(&mut game, stoat, &[bears]);
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.state.objects[bears as usize].zone, Zone::Graveyard);
    assert_eq!(game.state.objects[stoat as usize].damage, 0);
    assert_eq!(game.state.players[0].life, 22);
    let first = game.snapshot();
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::CombatDamage);
    assert_eq!(game.state.objects[stoat as usize].damage, 0);
    assert_eq!(game.state.players[0].life, 22);
    let normal = game.snapshot();
    game.restore(&first);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), normal);
}

#[test]
fn ordinary_attacker_survives_first_strike_then_deals_normal_damage() {
    let mut game = scenario();
    let giant = permanent(&mut game, Player::P0, "Hill Giant");
    let stoat = permanent(&mut game, Player::P1, "Brightblade Stoat");
    combat(&mut game, giant, &[stoat]);
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.state.objects[giant as usize].damage, 2);
    assert_eq!(game.state.players[1].life, 22);
    pass_pair(&mut game);
    assert_eq!(game.state.objects[giant as usize].zone, Zone::Battlefield);
    assert_eq!(game.state.objects[giant as usize].damage, 2);
    assert_eq!(game.state.objects[stoat as usize].zone, Zone::Graveyard);
    assert_eq!(game.state.players[1].life, 22); // No second hit from first strike.
}

#[test]
fn double_strike_assigns_twice_and_normal_damage_is_simultaneous() {
    let mut game = scenario();
    let cap = permanent(&mut game, Player::P0, "Raging Redcap");
    let bears = permanent(&mut game, Player::P1, "Grizzly Bears");
    combat(&mut game, cap, &[bears]);
    pass_pair(&mut game);
    assert_eq!(game.state.objects[bears as usize].damage, 1);
    assert_eq!(game.state.objects[cap as usize].damage, 0);
    pass_pair(&mut game);
    assert_eq!(game.state.objects[bears as usize].zone, Zone::Graveyard);
    assert_eq!(game.state.objects[cap as usize].zone, Zone::Graveyard);

    let mut game = scenario();
    let shrike = permanent(&mut game, Player::P0, "Shrike Force");
    combat(&mut game, shrike, &[]);
    pass_pair(&mut game);
    assert_eq!(game.state.players[1].life, 19);
    pass_pair(&mut game);
    assert_eq!(game.state.players[1].life, 18);
}

#[test]
fn first_step_decisions_and_priority_window_keep_full_continuations() {
    let mut game = scenario();
    let blade = permanent(&mut game, Player::P0, "Swiftblade Vindicator");
    let piker = permanent(&mut game, Player::P1, "Goblin Piker");
    let forest = permanent(&mut game, Player::P0, "Forest");
    let growth = spawn(&mut game, Player::P0, Card::GiantGrowth, Zone::Hand);
    combat(&mut game, blade, &[piker]);
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.state.objects[piker as usize].zone, Zone::Graveyard);
    let first = game.snapshot();
    act(&mut game, Action::TapMana { card: forest });
    cast(&mut game, growth, Some(Target::Card(blade)));
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    pass_pair(&mut game);
    assert_eq!(game.state.players[1].life, 16); // Blocked, but trample with no live blockers.
    game.restore(&first);
    pass_pair(&mut game);
    assert_eq!(game.state.players[1].life, 19);
}

#[test]
fn deathtouch_and_lifelink_use_last_known_source_and_noncombat_damage() {
    let mut game = scenario();
    let nighthawk = permanent(&mut game, Player::P0, "Vampire Nighthawk");
    let giant = permanent(&mut game, Player::P1, "Hill Giant");
    let source = game.damage_source(nighthawk);
    game.move_object(nighthawk, Zone::Graveyard);
    // A changed source slot cannot change the already captured attribution.
    game.state.objects[nighthawk as usize].card = Card::Forest;
    let gains = game.apply_damage_batch(&[DamageEvent {
        source,
        target: Target::Card(giant),
        amount: 1,
        combat: false,
    }]);
    assert_eq!(gains.len(), 1);
    assert_eq!(game.state.players[0].life, 21);
    game.state_based_actions();
    assert_eq!(game.state.objects[giant as usize].zone, Zone::Graveyard);
    let gains = game.apply_damage_batch(&[DamageEvent {
        source,
        target: Target::Player(Player::P1),
        amount: 0,
        combat: false,
    }]);
    assert!(gains.is_empty());
    assert_eq!(game.state.players[0].life, 21);
    assert_eq!(game.state.players[1].life, 20);
}

#[test]
fn simultaneous_lifelink_prevents_loss_even_when_its_source_dies() {
    let mut game = scenario();
    game.state.active = Player::P1;
    game.state.players[0].life = 1;
    let giant = permanent(&mut game, Player::P1, "Hill Giant");
    let piker = permanent(&mut game, Player::P1, "Goblin Piker");
    let soldier = permanent(&mut game, Player::P0, "Bishop's Soldier");
    combat(&mut game, giant, &[soldier]);
    game.state.attackers.push(Attacker {
        card: piker,
        blocked: false,
    });
    pass_pair(&mut game);
    assert_eq!(game.state.players[0].life, 1); // 1 - 2 + 2, all before SBA.
    assert_eq!(game.state.objects[soldier as usize].zone, Zone::Graveyard);
    assert!(game.outcome().is_none());
}

#[test]
fn lifelink_groups_simultaneous_targets_by_source_and_uses_wide_life_totals() {
    let mut game = scenario();
    let hawk = permanent(&mut game, Player::P0, "Healer's Hawk");
    let soldier = permanent(&mut game, Player::P0, "Bishop's Soldier");
    let giant = permanent(&mut game, Player::P1, "Hill Giant");
    game.state.players[0].life = i64::from(i32::MAX);
    let a = game.damage_source(hawk);
    let b = game.damage_source(soldier);
    let gains = game.apply_damage_batch(&[
        DamageEvent {
            source: a,
            target: Target::Card(giant),
            amount: 1,
            combat: true,
        },
        DamageEvent {
            source: a,
            target: Target::Player(Player::P1),
            amount: 2,
            combat: true,
        },
        DamageEvent {
            source: b,
            target: Target::Player(Player::P1),
            amount: 2,
            combat: true,
        },
    ]);
    assert_eq!(
        gains,
        vec![
            LifeGainEvent {
                source: a,
                amount: 3
            },
            LifeGainEvent {
                source: b,
                amount: 2
            }
        ]
    );
    assert_eq!(game.state.players[0].life, i64::from(i32::MAX) + 5);
}

#[test]
fn trample_honors_marked_damage_and_does_not_overflow_when_blocking() {
    let mut game = scenario();
    let dino = permanent(&mut game, Player::P0, "Colossadactyl");
    let bears = permanent(&mut game, Player::P1, "Grizzly Bears");
    game.state.objects[bears as usize].damage = 1;
    combat(&mut game, dino, &[bears]);
    pass_pair(&mut game);
    assert!(game.legal_actions().contains(&Action::AssignDamage {
        target: Target::Card(bears),
        amount: 1
    }));
    let before = game.snapshot();
    assert_eq!(
        game.apply(
            Player::P0,
            Action::AssignDamage {
                target: Target::Card(bears),
                amount: 0
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(game.snapshot(), before);
    act(
        &mut game,
        Action::AssignDamage {
            target: Target::Card(bears),
            amount: 1,
        },
    );
    assert_eq!(game.state.players[1].life, 17);

    let mut game = scenario();
    let piker = permanent(&mut game, Player::P0, "Goblin Piker");
    let dino = permanent(&mut game, Player::P1, "Colossadactyl");
    combat(&mut game, piker, &[dino]);
    pass_pair(&mut game);
    assert_eq!(game.state.players[0].life, 20);
    assert_eq!(game.state.players[1].life, 20);
}

#[test]
fn blocked_trample_deals_to_player_after_all_blockers_disappear() {
    let mut game = scenario();
    let dino = permanent(&mut game, Player::P0, "Colossadactyl");
    let piker = permanent(&mut game, Player::P1, "Goblin Piker");
    combat(&mut game, dino, &[piker]);
    game.move_object(piker, Zone::Graveyard);
    pass_pair(&mut game);
    assert_eq!(game.state.players[1].life, 16);
}

#[test]
fn exhaustive_small_trample_partitions_match_complete_rule_constraints() {
    // Compare full reachable assignments against a mathematical oracle. There is
    // no lethal-order requirement among blockers when no damage reaches player.
    fn enumerate(
        game: Game,
        b1: CardId,
        b2: CardId,
        power: i32,
        mut amounts: [Option<i32>; 2],
        out: &mut BTreeSet<(i32, i32, i32)>,
    ) {
        if game.state.pending != Pending::Damage {
            let player = (20 - game.state.players[1].life) as i32;
            let used: i32 = amounts.iter().flatten().sum();
            let mut left = power - used - player;
            for amount in &mut amounts {
                if amount.is_none() {
                    *amount = Some(left);
                    left = 0;
                }
            }
            assert_eq!(left, 0);
            out.insert((amounts[0].unwrap(), amounts[1].unwrap(), player));
            return;
        }
        let saved = game.snapshot();
        for action in game
            .legal_actions()
            .into_iter()
            .filter(|a| matches!(a, Action::AssignDamage { .. }))
        {
            let mut branch = game.clone();
            let Action::AssignDamage {
                target: Target::Card(id),
                amount,
            } = action
            else {
                panic!("unexpected target")
            };
            let mut next = amounts;
            next[if id == b1 {
                0
            } else {
                assert_eq!(id, b2);
                1
            }] = Some(amount);
            act(&mut branch, action);
            assert_eq!(game.snapshot(), saved);
            enumerate(branch, b1, b2, power, next, out);
        }
    }
    for deathtouch in [false, true] {
        for marked in 0..=1 {
            for power in 1..=6 {
                let mut game = scenario();
                let dino = permanent(&mut game, Player::P0, "Colossadactyl");
                game.state.objects[dino as usize].growth = (power - 4).into();
                let b1 = permanent(&mut game, Player::P1, "Grizzly Bears");
                let b2 = permanent(&mut game, Player::P1, "Goblin Piker");
                game.state.objects[b1 as usize].damage = marked;
                combat(&mut game, dino, &[b1, b2]);
                pass_pair(&mut game);
                if deathtouch {
                    let choice = game.state.damage_choices.last_mut().unwrap();
                    choice.source.keywords = choice.source.keywords.union(Keywords::DEATHTOUCH);
                }
                let mut actual = BTreeSet::new();
                enumerate(game, b1, b2, power, [None, None], &mut actual);
                let required1 = if deathtouch { 1 } else { 2 - marked };
                let mut expected = BTreeSet::new();
                for a in 0..=power {
                    for b in 0..=power - a {
                        let player = power - a - b;
                        if player == 0 || (a >= required1 && b >= 1) {
                            expected.insert((a, b, player));
                        }
                    }
                }
                assert_eq!(
                    actual, expected,
                    "power {power}, marked {marked}, deathtouch {deathtouch}"
                );
            }
        }
    }
}

#[test]
fn zero_power_does_not_generate_damage_or_lifelink_events() {
    let mut game = scenario();
    let archer = permanent(&mut game, Player::P0, "Aegis Turtle");
    combat(&mut game, archer, &[]);
    pass_pair(&mut game);
    assert_eq!(game.state.players[1].life, 20);
    assert!(game.state.hits.is_empty());
    assert!(game.state.damage_choices.is_empty());
}

#[test]
fn restore_during_first_strike_multiblocker_assignment_replays_exactly() {
    let mut game = scenario();
    let stoat = permanent(&mut game, Player::P0, "Brightblade Stoat");
    let b1 = permanent(&mut game, Player::P1, "Grizzly Bears");
    let b2 = permanent(&mut game, Player::P1, "Grizzly Bears");
    combat(&mut game, stoat, &[b1, b2]);
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.state.pending, Pending::Damage);
    assert_eq!(game.observe(Player::P0).damage_source, Some(stoat));
    assert!(game.observe(Player::P1).damage_assigned.is_empty());
    assert_eq!(game.observe(Player::P1).damage_source, None);
    let saved = game.snapshot();
    let action = Action::AssignDamage {
        target: Target::Card(b1),
        amount: 2,
    };
    act(&mut game, action.clone());
    let outcome = game.snapshot();
    assert_eq!(game.state.objects[b1 as usize].zone, Zone::Graveyard);
    assert_eq!(game.state.objects[b2 as usize].zone, Zone::Battlefield);
    assert_eq!(game.state.players[0].life, 22);
    game.restore(&saved);
    act(&mut game, action);
    assert_eq!(game.snapshot(), outcome);
    game.restore(&saved);
    act(
        &mut game,
        Action::AssignDamage {
            target: Target::Card(b1),
            amount: 0,
        },
    );
    assert_eq!(game.state.objects[b1 as usize].zone, Zone::Battlefield);
    assert_eq!(game.state.objects[b2 as usize].zone, Zone::Graveyard);
}

#[test]
fn deathtouch_allows_both_split_and_overassigned_damage_without_trample() {
    let mut game = scenario();
    let source = permanent(&mut game, Player::P0, "Vampire Nighthawk");
    let b1 = permanent(&mut game, Player::P1, "Magnigoth Sentry");
    let b2 = permanent(&mut game, Player::P1, "Thornweald Archer");
    combat(&mut game, source, &[b1, b2]);
    pass_pair(&mut game);
    let saved = game.snapshot();
    act(
        &mut game,
        Action::AssignDamage {
            target: Target::Card(b1),
            amount: 1,
        },
    );
    assert_eq!(game.state.objects[b1 as usize].zone, Zone::Graveyard);
    assert_eq!(game.state.objects[b2 as usize].zone, Zone::Graveyard);
    assert_eq!(game.state.objects[source as usize].zone, Zone::Graveyard);
    assert_eq!(game.state.players[0].life, 22);
    game.restore(&saved);
    act(
        &mut game,
        Action::AssignDamage {
            target: Target::Card(b1),
            amount: 0,
        },
    );
    assert_eq!(game.state.objects[b1 as usize].zone, Zone::Battlefield);
    assert_eq!(game.state.objects[b2 as usize].zone, Zone::Graveyard);
    assert_eq!(game.state.players[0].life, 22);
}
