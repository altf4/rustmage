//! Individual Kingpin rules: Extort payments and combat assignment without changing power.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::msh_thirteenth_combined_tests::owned_checked_step;
use super::*;
const KING: &str = "The Kingpin of Crime";
fn game(p: Player) -> Game {
    let mut g = setup(p);
    for s in &mut g.state.players {
        s.mana = [0; 6]
    }
    g
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass)
    }
    assert_eq!(g.actor(), Some(p));
}
fn seed(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(Card::from_name(name).unwrap().is_supported(), "{name}");
    named(g, p, name, zone)
}

fn packet(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = seed(g, p, name, Zone::Hand);
    let (mut mana, generic) = Card::from_name(name).unwrap().cost();
    mana[5] += generic;
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = packet(g, p, name, targets);
    settle(g);
    id
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let b = g.state_bytes();
    let views = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.state_bytes(), b);
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
        views
    );
}
fn orphan(g: Game, a: Action) -> Game {
    let p = g.actor().unwrap();
    let i = g.legal_actions().iter().position(|x| *x == a).unwrap();
    owned_checked_step(g, p, i, a)
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..240 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::AssignDamage { .. }
                        | Action::ChooseKicker { pay: false }
                        | Action::DeclineResolutionMana
                        | Action::ChooseOptional { accept: false }
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, a)
    }
    panic!("stalled")
}
fn main(g: &mut Game, p: Player) {
    until(g, |g| {
        g.state.step == Step::Main1 && g.state.active == p && g.state.pending == Pending::Priority
    });
    priority(g, p);
}
fn extort(g: &mut Game, p: Player, pool: Mana) -> CardId {
    priority(g, p);
    g.state.players[p.index()].mana = pool;
    let c = named(g, p, "Mox Jasper", Zone::Hand);
    cast(g, c, None, &[]);
    until(g, |g| {
        g.legal_actions().contains(&Action::DeclineResolutionMana)
    });
    c
}
fn pay(g: &mut Game, payment: Mana) {
    checked(g, Action::PayResolutionMana { payment });
    while matches!(g.state.pending, Pending::ManaLotSpend { .. }) {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseManaLotSpend { .. }))
            .unwrap();
        checked(g, a)
    }
}
fn ready(p: Player) -> (Game, CardId) {
    let mut g = game(p);
    let k = paid(&mut g, p, KING, &[]);
    next_turn(&mut g);
    next_turn(&mut g);
    main(&mut g, p);
    (g, k)
}
fn attack(g: &mut Game, ids: &[CardId], accept: bool) {
    until(g, |g| g.state.pending == Pending::Attackers);
    for &card in ids {
        checked(g, Action::Attack { card });
    }
    checked(g, Action::FinishAttackers);
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
    checked(g, Action::ChooseOptional { accept });
}
fn endcombat(g: &mut Game) {
    until(g, |g| {
        g.state.step == Step::EndCombat && g.state.pending == Pending::Priority
    });
}
#[test]
fn msh_kingpin_paid_body_self_cast_does_not_extort_and_both_hybrid_choices_pay_once() {
    for p in [Player::P0, Player::P1] {
        for color in [0, 2] {
            let mut g = game(p);
            let k = paid(&mut g, p, KING, &[]);
            assert_eq!(
                g.stats(k),
                Stats {
                    power: 1,
                    toughness: 5
                }
            );
            assert_eq!(g.state.players[p.index()].life, 20);
            assert_eq!(g.observe(p).toughness_combat_damage, None);
            let mut pool = [0; 6];
            pool[color] = 2;
            extort(&mut g, p, pool);
            let mut payment = [0; 6];
            payment[color] = 1;
            reject(&mut g, Action::PayResolutionMana { payment: pool });
            g = orphan(g, Action::PayResolutionMana { payment });
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 21);
            assert_eq!(g.state.players[p.other().index()].life, 19);
            assert_eq!(g.state.players[p.index()].mana, payment);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::PayResolutionMana { .. }))
            );
        }
    }
}
#[test]
fn msh_kingpin_extort_decline_and_wrong_color_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        paid(&mut g, p, KING, &[]);
        extort(&mut g, p, [0, 1, 0, 0, 0, 1]);
        reject(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 1, 0, 0, 0, 0],
            },
        );
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn msh_kingpin_extort_queued_source_departure_keeps_captured_controller_and_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let k = paid(&mut g, p, KING, &[]);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
        let c = named(&mut g, p, "Mox Jasper", Zone::Hand);
        cast(&mut g, c, None, &[]);
        until(&mut g, |g| {
            g.state.pending == Pending::Priority
                && g.state
                    .stack
                    .iter()
                    .any(|i| i.card == k && matches!(i.kind, StackKind::TriggeredAbility { .. }))
        });
        packet(&mut g, p.other(), "Murder", &[Target::Card(k)]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::DeclineResolutionMana)
        });
        assert_eq!(g.object_view(k).zone, Zone::Graveyard);
        pay(&mut g, [1, 0, 0, 0, 0, 0]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}
#[test]
fn msh_kingpin_extort_actual_gain_addition_and_prohibition_use_normal_event_pipeline() {
    for p in [Player::P0, Player::P1] {
        for no_gain in [false, true] {
            let mut g = game(p);
            paid(&mut g, p, KING, &[]);
            let extra = if no_gain {
                "Giant Cindermaw"
            } else {
                "Angel of Vitality"
            };
            paid(&mut g, p, extra, &[]);
            extort(&mut g, p, [0, 0, 1, 0, 0, 0]);
            pay(&mut g, [0, 0, 1, 0, 0, 0]);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if no_gain { 20 } else { 22 }
            );
            assert_eq!(g.state.players[p.other().index()].life, 19);
        }
    }
}
#[test]
fn msh_kingpin_extort_player_hexproof_does_not_stop_untargeted_drain() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        paid(&mut g, p, KING, &[]);
        next_turn(&mut g);
        main(&mut g, p.other());
        let cap = paid(&mut g, p.other(), "Captain America, Super-Soldier", &[]);
        assert_eq!(g.counter_count(cap, CounterKind::Shield), 1);
        assert!(g.player_has_hexproof(p.other()));
        next_turn(&mut g);
        main(&mut g, p);
        extort(&mut g, p, [1, 0, 0, 0, 0, 0]);
        pay(&mut g, [1, 0, 0, 0, 0, 0]);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}
#[test]
fn msh_kingpin_declared_attack_payment_changes_assignment_not_actual_power() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let (mut g, k) = ready(p);
            attack(&mut g, &[k], accept);
            assert_eq!(
                g.stats(k),
                Stats {
                    power: 1,
                    toughness: 5
                }
            );
            assert_eq!(g.combat_assignment_amount(k), if accept { 5 } else { 1 });
            if accept {
                assert_eq!(
                    g.observe(p).toughness_combat_damage,
                    Some(if p == Player::P0 {
                        [true, false]
                    } else {
                        [false, true]
                    })
                );
            } else {
                assert_eq!(g.observe(p).toughness_combat_damage, None);
            }
            endcombat(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if accept { 18 } else { 20 }
            );
            assert_eq!(
                g.state.players[p.other().index()].life,
                if accept { 15 } else { 19 }
            );
            next_turn(&mut g);
            assert_eq!(g.observe(p).toughness_combat_damage, None);
            assert_eq!(g.combat_assignment_amount(k), 1);
        }
    }
}
#[test]
fn msh_kingpin_attack_with_other_creatures_triggers_once_without_kingpin_attacking() {
    for p in [Player::P0, Player::P1] {
        let (mut g, k) = ready(p);
        let a = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let b = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        attack(&mut g, &[a, b], true);
        assert!(!g.object_view(k).tapped);
        assert_eq!(g.state.players[p.index()].life, 18);
        endcombat(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 15);
    }
}
#[test]
fn msh_kingpin_attack_life_payment_affordability_and_owned_pending_choice() {
    for p in [Player::P0, Player::P1] {
        let (mut g, k) = ready(p);
        g.state.players[p.index()].life = 1;
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: k });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        assert_eq!(g.observe(p).toughness_combat_damage, None);
        endcombat(&mut g);
        assert_eq!(g.state.players[p.index()].life, 1);
        let (mut g, k) = ready(p);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: k });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        g = orphan(g, Action::ChooseOptional { accept: true });
        assert_eq!(g.state.players[p.index()].life, 18);
        endcombat(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 15);
    }
}
#[test]
fn msh_kingpin_rule_survives_source_departure_and_applies_to_later_controlled_creatures() {
    for p in [Player::P0, Player::P1] {
        let (mut g, k) = ready(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        attack(&mut g, &[bear], true);
        packet(&mut g, p.other(), "Murder", &[Target::Card(k)]);
        settle(&mut g);
        assert_eq!(g.object_view(k).zone, Zone::Graveyard);
        let later = named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
        assert_eq!(g.combat_assignment_amount(later), 5);
        assert_eq!(g.stats(later).power, 0);
        endcombat(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn msh_kingpin_rule_uses_current_stats_and_current_controller_with_signed_values() {
    for p in [Player::P0, Player::P1] {
        let (mut g, k) = ready(p);
        attack(&mut g, &[k], true);
        for (power, toughness, expected) in
            [(1, 5, 5), (6, 5, 6), (5, 5, 5), (-2, 3, 3), (-3, -1, 0)]
        {
            g.state.objects[k as usize].growth = Stats {
                power: power - 1,
                toughness: toughness - 5,
            };
            assert_eq!(g.combat_assignment_amount(k), expected);
            assert_eq!(g.stats(k), Stats { power, toughness });
        }
        g.state.objects[k as usize].growth = Stats::default();
        let own = named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
        let other = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        assert_eq!(g.combat_assignment_amount(own), 5);
        assert_eq!(g.combat_assignment_amount(other), 0);
    }
}
#[test]
fn msh_kingpin_extort_does_not_trigger_for_land_or_opponent_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        paid(&mut g, p, KING, &[]);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: land });
        assert!(g.state.pending_triggers.is_empty());
        paid(&mut g, p.other(), "Shock", &[Target::Player(p)]);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn msh_kingpin_extort_cannot_spend_restricted_equipment_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        paid(&mut g, p, KING, &[]);
        let r = paid(&mut g, p, "Ronin, Shadow Stalker", &[]);
        checked(
            &mut g,
            Action::AnnounceMana {
                card: r,
                ability: 0,
                counter_payment: Counters::default(),
                production: [2, 0, 0, 0, 0, 0],
            },
        );
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, a);
        }
        let c = named(&mut g, p, "Mox Jasper", Zone::Hand);
        cast(&mut g, c, None, &[]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::DeclineResolutionMana)
        });
        reject(
            &mut g,
            Action::PayResolutionMana {
                payment: [1, 0, 0, 0, 0, 0],
            },
        );
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn msh_kingpin_real_copied_extort_instances_each_offer_one_independent_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let k = paid(&mut g, p, KING, &[]);
        let t = paid(&mut g, p, "Taskmaster, Mercenary Mimic", &[]);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == t)
                && g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(k),
                })
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(k),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        settle(&mut g);
        assert_eq!(
            g.stats(t),
            Stats {
                power: 1,
                toughness: 5
            }
        );
        priority(&mut g, p);
        extort(&mut g, p, [2, 0, 0, 0, 0, 0]);
        pay(&mut g, [1, 0, 0, 0, 0, 0]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::DeclineResolutionMana)
        });
        pay(&mut g, [1, 0, 0, 0, 0, 0]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn msh_kingpin_actual_first_strike_uses_toughness_before_ordinary_retaliation() {
    for p in [Player::P0, Player::P1] {
        let (mut g, k) = ready(p);
        let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        attack(&mut g, &[k], true);
        paid(&mut g, p, "Sure Strike", &[Target::Card(k)]);
        assert_eq!(
            g.stats(k),
            Stats {
                power: 4,
                toughness: 5
            }
        );
        assert_eq!(g.combat_assignment_amount(k), 5);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: k,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        endcombat(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.object_view(k).damage, 0);
    }
}
#[test]
fn msh_kingpin_actual_double_strike_assigns_toughness_in_both_damage_steps() {
    for p in [Player::P0, Player::P1] {
        let (mut g, k) = ready(p);
        attack(&mut g, &[k], true);
        paid(&mut g, p, "Twinblade Blessing", &[Target::Card(k)]);
        assert_eq!(g.stats(k).power, 1);
        endcombat(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 10);
    }
}
#[test]
fn msh_kingpin_actual_trample_assignment_uses_toughness_before_damage_replacements() {
    for p in [Player::P0, Player::P1] {
        let (mut g, k) = ready(p);
        paid(&mut g, p, "Overrun", &[]);
        let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        attack(&mut g, &[k], true);
        assert_eq!(
            g.stats(k),
            Stats {
                power: 4,
                toughness: 8
            }
        );
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: k,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        endcombat(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.other().index()].life, 14);
    }
}
#[test]
fn msh_kingpin_combat_rule_does_not_change_real_noncombat_power_damage() {
    for p in [Player::P0, Player::P1] {
        let (mut g, k) = ready(p);
        let victim = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        attack(&mut g, &[k], true);
        paid(
            &mut g,
            p,
            "Bite Down",
            &[Target::Card(k), Target::Card(victim)],
        );
        assert_eq!(g.object_view(victim).damage, 1);
        assert_eq!(g.combat_assignment_amount(k), 5);
        assert_eq!(g.stats(k).power, 1);
        endcombat(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 15);
    }
}
#[test]
fn msh_kingpin_time_stop_expires_paid_rule_at_real_cleanup() {
    for p in [Player::P0, Player::P1] {
        let (mut g, k) = ready(p);
        attack(&mut g, &[k], true);
        assert!(g.observe(p).toughness_combat_damage.is_some());
        paid(&mut g, p, "Time Stop", &[]);
        assert_eq!(g.observe(p).toughness_combat_damage, None);
        assert_eq!(g.combat_assignment_amount(k), 1);
        assert_eq!(g.state.active, p.other());
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn msh_kingpin_real_land_payment_requires_printed_white_black_and_generic() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let k = seed(&mut g, p, KING, Zone::Hand);
        let mut lands = Vec::new();
        for _ in 0..3 {
            lands.push(named(&mut g, p, "Forest", Zone::Battlefield));
        }
        reject(&mut g, Action::AnnounceCast { card: k });
        let w = named(&mut g, p, "Plains", Zone::Battlefield);
        let b = named(&mut g, p, "Swamp", Zone::Battlefield);
        let before = g.snapshot();
        let views = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: k })
        );
        assert_eq!(g.snapshot(), before);
        assert_eq!(
            [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
            views
        );
        checked(&mut g, Action::AnnounceCast { card: k });
        for card in [w, b, lands[0]] {
            checked(&mut g, Action::TapMana { card });
        }
        checked(
            &mut g,
            Action::PayCosts {
                payment: [1, 0, 1, 0, 1, 0],
            },
        );
        settle(&mut g);
        assert_eq!(
            g.stats(k),
            Stats {
                power: 1,
                toughness: 5
            }
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
#[test]
fn msh_kingpin_extort_disappears_with_ability_removal_and_restores_after_aura_dies() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let k = paid(&mut g, p, KING, &[]);
        let witness = paid(&mut g, p, "Witness Protection", &[Target::Card(k)]);
        let c = named(&mut g, p, "Mox Jasper", Zone::Hand);
        cast(&mut g, c, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert!(g.state.pending_triggers.is_empty());
        paid(&mut g, p, "Disenchant", &[Target::Card(witness)]);
        extort(&mut g, p, [1, 0, 0, 0, 0, 0]);
        pay(&mut g, [1, 0, 0, 0, 0, 0]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}
#[test]
fn msh_kingpin_extort_gains_life_before_terminal_state_based_loss() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        paid(&mut g, p, KING, &[]);
        g.state.players[p.other().index()].life = 1;
        extort(&mut g, p, [0, 0, 1, 0, 0, 0]);
        pay(&mut g, [0, 0, 1, 0, 0, 0]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.other().index()].life, 0);
        assert_eq!(g.outcome(), Some(Outcome::Winner(p)));
    }
}
