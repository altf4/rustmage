//! Paid full-card actions with independent owned/raw/index replay at every choice.
use super::ability_timeline_tests::{cast, checked, next_turn, settle, setup};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    super::tests::spawn(g, p, Card::from_name(name).unwrap(), zone)
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
fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, action);
}
fn cast_x(g: &mut Game, id: CardId, x: u32, target: CardId) {
    checked(g, Action::AnnounceCast { card: id });
    checked(g, Action::ChooseX { value: x });
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
}
fn activate_x(g: &mut Game, id: CardId, x: u32) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    assert!(g.observe(g.actor().unwrap().other()).x_choice.is_none());
    checked(g, Action::ChooseX { value: x });
    pay(g);
}
fn attack(g: &mut Game, ids: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &id in ids {
        checked(g, Action::Attack { card: id });
    }
    checked(g, Action::FinishAttackers);
}
#[test]
fn combat_allocation_biogenic_declares_positive_shares_then_doubles_only_plus_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[a as usize].counters.plus_one = 1;
        g.state.objects[a as usize].counters.stun = 2;
        let spell = named(&mut g, p, "Biogenic Upgrade", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 6, 0];
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, Action::FinishTargets);
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
        checked(&mut g, Action::AllocateTargetAmount { amount: 2 });
        checked(&mut g, Action::AllocateTargetAmount { amount: 1 });
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 6);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(a, CounterKind::Stun), 2);
    }
}
#[test]
fn combat_allocation_biogenic_replacements_and_illegal_target_keep_original_distribution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[a as usize].counters.plus_one = 1;
        let spell = named(&mut g, p, "Biogenic Upgrade", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
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
        checked(&mut g, Action::AllocateTargetAmount { amount: 2 });
        checked(&mut g, Action::AllocateTargetAmount { amount: 1 });
        pay(&mut g);
        g.move_object(b, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 15);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
    }
}
#[test]
fn combat_allocation_drakuseth_paid_attack_three_distinct_targets_and_source_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let drak = named(&mut g, p, "Drakuseth, Maw of Flames", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 7, 0, 0];
        cast(&mut g, drak, None, &[]);
        settle(&mut g);
        g.state.objects[drak as usize].summoning_sick = false;
        let a = named(&mut g, p.other(), "Colossal Dreadmaw", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Colossal Dreadmaw", Zone::Battlefield);
        attack(&mut g, &[drak]);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
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
        g.move_object(drak, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert_eq!(g.state.objects[a as usize].damage, 3);
        assert_eq!(g.state.objects[b as usize].damage, 3);
    }
}
#[test]
fn combat_allocation_drakuseth_optional_other_targets_and_partial_illegality() {
    for extra in [false, true] {
        let mut g = setup(Player::P0);
        let drak = named(
            &mut g,
            Player::P0,
            "Drakuseth, Maw of Flames",
            Zone::Battlefield,
        );
        let a = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        attack(&mut g, &[drak]);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        if extra {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(Player::P1),
                },
            );
        }
        checked(&mut g, Action::FinishTargets);
        g.move_object(a, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.state.players[1].life, if extra { 17 } else { 20 });
    }
}
#[test]
fn combat_allocation_goblin_uses_actual_excess_damage_marked_damage_and_prevention() {
    for p in [Player::P0, Player::P1] {
        for prevent in [false, true] {
            let mut g = setup(p);
            if prevent {
                named(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
            }
            let target = named(&mut g, p.other(), "Colossal Dreadmaw", Zone::Battlefield);
            g.state.objects[target as usize].damage = 2;
            let spell = named(&mut g, p, "Goblin Negotiation", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 9, 0, 0];
            cast_x(&mut g, spell, 7, target);
            settle(&mut g);
            let tokens = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.card == Card::from_name("@token/e58ebd913e8be25509b853b8").unwrap()
                })
                .count();
            assert_eq!(tokens, if prevent { 0 } else { 3 });
            assert_eq!(g.on_battlefield(target), prevent);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn combat_allocation_goblin_damage_doubling_and_token_multiplication_are_separate() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, "Twinflame Tyrant", Zone::Battlefield);
    named(&mut g, Player::P0, "Doubling Season", Zone::Battlefield);
    let target = named(&mut g, Player::P1, "Colossal Dreadmaw", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Goblin Negotiation", Zone::Hand);
    cast_x(&mut g, spell, 5, target);
    settle(&mut g);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield
                && o.card == Card::from_name("@token/e58ebd913e8be25509b853b8").unwrap())
            .count(),
        8
    );
}
#[test]
fn combat_allocation_aurelia_paid_first_attack_untaps_then_adds_combat_without_main_phase() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let aurelia = named(&mut g, p, "Aurelia, the Warleader", Zone::Hand);
        g.state.players[p.index()].mana = [2, 0, 0, 4, 0, 0];
        cast(&mut g, aurelia, None, &[]);
        settle(&mut g);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        attack(&mut g, &[aurelia, bear]);
        settle(&mut g);
        assert!(!g.state.objects[bear as usize].tapped);
        assert_eq!(g.observe(p).additional_combats, 1);
        // Real phase advancement; skip combat damage to keep the fixture nonterminal.
        g.state.step = Step::EndCombat;
        g.give_priority();
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.step, Step::BeginCombat);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Attack { card: aurelia });
        checked(&mut g, Action::Attack { card: bear });
        checked(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert!(g.state.objects[bear as usize].tapped);
        assert_eq!(g.observe(p).additional_combats, 0);
        g.state.step = Step::EndCombat;
        g.give_priority();
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.step, Step::Main2);
    }
}
#[test]
fn combat_allocation_aurelia_prior_attack_without_ability_does_not_trigger_late() {
    let mut g = setup(Player::P0);
    let a = named(
        &mut g,
        Player::P0,
        "Aurelia, the Warleader",
        Zone::Battlefield,
    );
    let reference = g.object_ref(a);
    g.state.attacked_this_turn.push(reference);
    attack(&mut g, &[a]);
    settle(&mut g);
    assert_eq!(g.observe(Player::P0).additional_combats, 0);
}
#[test]
fn combat_allocation_joraga_maximizes_requirements_and_does_not_affect_later_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let block = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Joraga Invocation", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 6, 0];
        cast(&mut g, spell, None, &[]);
        settle(&mut g);
        let later = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(g.stats(a).power, 5);
        assert_eq!(g.stats(later).power, 2);
        attack(&mut g, &[a, b, later]);
        settle(&mut g);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.pending, Pending::Blockers);
        reject(&mut g, Action::FinishBlockers);
        reject(
            &mut g,
            Action::Block {
                blocker: block,
                attacker: later,
            },
        );
        checked(
            &mut g,
            Action::Block {
                blocker: block,
                attacker: b,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        next_turn(&mut g);
        assert!(g.observe(p).must_be_blocked.is_empty());
        assert_eq!(g.stats(a).power, 2);
    }
}
#[test]
fn combat_allocation_joraga_restrictions_and_two_blocker_bundles_have_no_dead_end() {
    let mut g = setup(Player::P0);
    let a = named(&mut g, Player::P0, "Crypt Feaster", Zone::Battlefield);
    let b = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let blocker = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Joraga Invocation", Zone::Hand);
    cast(&mut g, spell, None, &[]);
    settle(&mut g);
    attack(&mut g, &[a, b]);
    settle(&mut g);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    reject(
        &mut g,
        Action::Block {
            blocker,
            attacker: a,
        },
    );
    checked(
        &mut g,
        Action::Block {
            blocker,
            attacker: b,
        },
    );
    checked(&mut g, Action::FinishBlockers);
}
#[test]
fn combat_allocation_hellkite_paid_x_destroys_current_nonlands_once_even_after_source_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hellkite = named(&mut g, p, "Steel Hellkite", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 6];
        cast(&mut g, hellkite, None, &[]);
        settle(&mut g);
        let source = g.damage_source(hellkite);
        g.apply_damage_batch(&[DamageEvent {
            source,
            target: Target::Player(p.other()),
            amount: 5,
            combat: true,
            preventable: true,
        }]);
        let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        activate_x(&mut g, hellkite, 2);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: hellkite,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        g.move_object(hellkite, Zone::Hand);
        settle(&mut g);
        assert!(!g.on_battlefield(victim));
        assert!(g.on_battlefield(own));
        assert!(g.on_battlefield(land));
    }
}
#[test]
fn combat_allocation_hellkite_zero_x_no_damage_is_legal_and_boost_repeats() {
    let mut g = setup(Player::P0);
    let h = named(&mut g, Player::P0, "Steel Hellkite", Zone::Battlefield);
    for _ in 0..2 {
        g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: h,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
    }
    assert_eq!(g.stats(h).power, 7);
    g.state.players[0].mana = [0; 6];
    activate_x(&mut g, h, 0);
    settle(&mut g);
    assert!(g.on_battlefield(h));
    next_turn(&mut g);
    assert_eq!(g.stats(h).power, 5);
    assert!(g.observe(Player::P0).combat_damaged_players.is_empty());
}
