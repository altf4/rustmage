//! Aggregate instructions consume all independently revalidated target roles.
//! Immutable fixture overlays admit no ordinary card or synthetic catalogue ID.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
const fn object(kinds: u8, relation: i8) -> TargetRule {
    TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds,
        relation,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }
}
const EXCHANGE_PLAN: TargetPlan = TargetPlan::Groups(&[
    TargetSpec::one(object(17, 0)),
    TargetSpec::one(object(17, 1)),
]);
const REWARD_PLAN: TargetPlan = TargetPlan::Groups(&[
    TargetSpec {
        rule: object(1, 0),
        min: 0,
        max: 1,
    },
    TargetSpec {
        rule: object(1, 1),
        min: 0,
        max: 1,
    },
]);
const REWARD_PROGRAM: &[Instruction] =
    &[Instruction::DestroyTargetsThenSourceCountersIfOwnCreature {
        kind: CounterKind::PlusOne,
        amount: 2,
    }];
const EXCHANGE: CardRules = CardRules {
    effect: Effect::Spell(EXCHANGE_PLAN, &[Instruction::ExchangeControlTargets]),
    ..CardRules::EMPTY
};
const SELECTED_EXCHANGE: CardRules = CardRules {
    effect: Effect::Spell(
        EXCHANGE_PLAN,
        &[
            Instruction::SelectGroup(1),
            Instruction::ExchangeControlTargets,
        ],
    ),
    ..CardRules::EMPTY
};
const fn activation(targets: TargetPlan, program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
const REWARD: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    activated: &[activation(REWARD_PLAN, REWARD_PROGRAM)],
    ..CardRules::EMPTY
};
const SELECTED_REWARD: CardRules = CardRules {
    activated: &[activation(
        REWARD_PLAN,
        &[
            Instruction::SelectGroup(1),
            Instruction::DestroyTargetsThenSourceCountersIfOwnCreature {
                kind: CounterKind::PlusOne,
                amount: 2,
            },
        ],
    )],
    ..REWARD
};
const LOCAL: CardRules = CardRules {
    activated: &[activation(
        REWARD_PLAN,
        &[Instruction::SelectGroup(1), Instruction::Destroy],
    )],
    ..REWARD
};
fn activate(g: &mut Game, source: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
fn target(g: &mut Game, id: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
}
fn done(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
}
fn drain(g: &mut Game) {
    for _ in 0..48 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("aggregate resolution bound");
}
fn exchange(g: &mut Game, p: Player, ours: CardId, theirs: CardId) -> CardId {
    let spell = put(g, p, "Divination", Zone::Hand);
    act(g, Action::AnnounceCast { card: spell });
    target(g, ours);
    target(g, theirs);
    done(g);
    spell
}
fn reward(g: &mut Game, source: CardId, ours: Option<CardId>, theirs: Option<CardId>) {
    activate(g, source);
    if let Some(id) = ours {
        target(g, id);
    } else {
        act(g, Action::FinishTargets);
    }
    if let Some(id) = theirs {
        target(g, id);
    } else {
        act(g, Action::FinishTargets);
    }
    done(g);
}
#[test]
fn relation_group_exchange_consumes_both_declared_roles_and_preserves_physical_owners() {
    for rules in [&EXCHANGE, &SELECTED_EXCHANGE] {
        let _scope = fixture_rules::activate(&[(Card::Divination, rules)]);
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
            let spell = exchange(&mut g, p, own, enemy);
            assert_eq!(
                g.state
                    .stack
                    .last()
                    .unwrap()
                    .targets
                    .iter()
                    .map(|b| b.group)
                    .collect::<Vec<_>>(),
                vec![0, 1]
            );
            drain(&mut g);
            assert_eq!(g.object_view(own).controller, p.other());
            assert_eq!(g.object_view(enemy).controller, p);
            assert_eq!(g.object_view(own).owner, p);
            assert_eq!(g.object_view(enemy).owner, p.other());
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            g.finish_cleanup();
            assert_eq!(g.object_view(own).controller, p.other());
            assert_eq!(g.object_view(enemy).controller, p);
        }
    }
}
#[test]
fn relation_group_exchange_rejects_wrong_role_selections_and_incomplete_pair_transactionally() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &EXCHANGE)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let otherown = put(&mut g, p, "Short Bow", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let land = put(&mut g, p, "Forest", Zone::Battlefield);
    let spell = put(&mut g, p, "Divination", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: spell });
    for id in [enemy, land] {
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    reject(&mut g, Action::FinishTargets);
    target(&mut g, own);
    for id in [own, otherown] {
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    reject(&mut g, Action::FinishTargets);
    target(&mut g, enemy);
    done(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(otherown).controller, p);
}
#[test]
fn exchange_is_all_or_neither_when_one_independent_controller_role_becomes_illegal() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &EXCHANGE)]);
    for p in [Player::P0, Player::P1] {
        for flip_own in [false, true] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
            exchange(&mut g, p, own, enemy);
            let changed = if flip_own { own } else { enemy };
            let controller = if flip_own { p.other() } else { p };
            g.change_controller(g.object_ref(changed), controller);
            drain(&mut g);
            assert_eq!(g.object_view(own).controller, controller);
            assert_eq!(g.object_view(enemy).controller, controller);
        }
    }
}
#[test]
fn exchange_does_not_reverse_a_prior_control_swap_when_both_original_roles_are_illegal() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &EXCHANGE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        let spell = exchange(&mut g, p, own, enemy);
        g.change_controller(g.object_ref(own), p.other());
        g.change_controller(g.object_ref(enemy), p);
        drain(&mut g);
        assert_eq!(g.object_view(own).controller, p.other());
        assert_eq!(g.object_view(enemy).controller, p);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}
#[test]
fn exchange_never_changes_the_survivor_if_other_target_leaves_or_reenters() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &EXCHANGE)]);
    for blink in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        exchange(&mut g, p, own, enemy);
        g.move_object(enemy, Zone::Hand);
        if blink {
            g.move_object(enemy, Zone::Battlefield);
        }
        drain(&mut g);
        assert_eq!(g.object_view(own).controller, p);
        assert_eq!(g.object_view(enemy).controller, p.other());
    }
}
#[test]
fn reward_aggregate_destroys_both_groups_and_rewards_controlled_destruction_once_regardless_of_active_group()
 {
    for rules in [&REWARD, &SELECTED_REWARD] {
        let _scope = fixture_rules::activate(&[(Card::HillGiant, rules)]);
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            reward(&mut g, source, Some(own), Some(enemy));
            drain(&mut g);
            for id in [own, enemy] {
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            }
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
            assert_eq!(stats(&g, source), (6, 5));
        }
    }
}
#[test]
fn reward_revalidates_each_control_role_independently_and_keeps_legal_partial_destruction() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &REWARD)]);
    for p in [Player::P0, Player::P1] {
        for flips in [1, 2, 3] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            reward(&mut g, source, Some(own), Some(enemy));
            if flips & 1 != 0 {
                g.change_controller(g.object_ref(own), p.other());
            }
            if flips & 2 != 0 {
                g.change_controller(g.object_ref(enemy), p);
            }
            drain(&mut g);
            assert_eq!(
                g.object_view(own).zone,
                if flips & 1 != 0 {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.object_view(enemy).zone,
                if flips & 2 != 0 {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                if flips == 2 { 2 } else { 0 }
            );
        }
    }
}
#[test]
fn optional_role_groups_may_skip_either_player_and_cannot_select_another_players_creature() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &REWARD)]);
    for only_own in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        activate(&mut g, source);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        if only_own {
            target(&mut g, own);
        } else {
            act(&mut g, Action::FinishTargets);
        }
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        if only_own {
            act(&mut g, Action::FinishTargets);
        } else {
            target(&mut g, enemy);
        }
        done(&mut g);
        drain(&mut g);
        assert_eq!(
            g.object_view(own).zone,
            if only_own {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            g.object_view(enemy).zone,
            if only_own {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(
            g.counter_count(source, CounterKind::PlusOne),
            if only_own { 2 } else { 0 }
        );
    }
}
#[test]
fn aggregate_reward_requires_actual_controlled_destruction_and_still_destroys_the_other_legal_group()
 {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
        }
    }
    let _reset = Reset;
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &REWARD)]);
    for protect_own in [0, 1, 2] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        if protect_own == 0 {
            g.state.objects[enemy as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        } else if protect_own == 1 {
            g.state.objects[own as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        } else {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(Some(g.object_ref(own))));
        }
        reward(&mut g, source, Some(own), Some(enemy));
        drain(&mut g);
        assert_eq!(
            g.object_view(own).zone,
            if protect_own == 0 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            g.object_view(enemy).zone,
            if protect_own == 0 {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(
            g.counter_count(source, CounterKind::PlusOne),
            if protect_own == 0 { 2 } else { 0 }
        );
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
    }
}
#[test]
fn ordinary_select_group_destruction_remains_scoped_to_its_active_group() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &LOCAL)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    reward(&mut g, source, Some(own), Some(enemy));
    drain(&mut g);
    assert_eq!(g.object_view(own).zone, Zone::Battlefield);
    assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
}
#[test]
fn aggregate_exchange_owned_raw_index_resolution_and_restored_role_change_branch_are_independent() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &EXCHANGE)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
    exchange(&mut g, p, own, enemy);
    g.state.priority = p.other();
    g.state.passes = 1;
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let i = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == Action::Pass)
        .unwrap();
    act(&mut raw, Action::Pass);
    indexed
        .apply_index(p.other(), indexed.revision(), i)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let expected = raw.snapshot();
    indexed.restore(&saved).unwrap();
    drop(g);
    act(&mut indexed, Action::Pass);
    assert_eq!(indexed.snapshot(), expected);
    let mut changed = indexed.clone();
    changed.restore(&saved).unwrap();
    changed.change_controller(changed.object_ref(own), p.other());
    drain(&mut changed);
    assert_eq!(changed.object_view(own).controller, p.other());
    assert_eq!(changed.object_view(enemy).controller, p.other());
    assert_eq!(raw.object_view(enemy).controller, p);
}
#[test]
fn aggregate_destruction_owned_raw_index_restore_keeps_exact_targets_and_partial_outcome() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &REWARD)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    reward(&mut g, source, Some(own), Some(enemy));
    g.state.priority = p.other();
    g.state.passes = 1;
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let i = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == Action::Pass)
        .unwrap();
    act(&mut raw, Action::Pass);
    indexed
        .apply_index(p.other(), indexed.revision(), i)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    drop(g);
    indexed.change_controller(indexed.object_ref(own), p.other());
    drain(&mut indexed);
    assert_eq!(indexed.object_view(own).zone, Zone::Battlefield);
    assert_eq!(indexed.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(indexed.counter_count(source, CounterKind::PlusOne), 0);
    assert_eq!(raw.counter_count(source, CounterKind::PlusOne), 2);
}

#[test]
fn reward_uses_the_frozen_ability_controller_after_source_control_change_and_exact_source_identity()
{
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &REWARD)]);
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            reward(&mut g, source, Some(own), Some(enemy));
            assert_eq!(g.state.stack.last().unwrap().controller, p);
            if blink {
                g.move_object(source, Zone::Hand);
                g.move_object(source, Zone::Battlefield);
            } else {
                g.change_controller(g.object_ref(source), p.other());
            }
            drain(&mut g);
            for id in [own, enemy] {
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            }
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                if blink { 0 } else { 2 }
            );
            assert_eq!(
                g.object_view(source).controller,
                if blink { p } else { p.other() }
            );
        }
    }
}

#[test]
fn aggregate_destruction_is_one_simultaneous_batch_and_deduplicates_repeated_group_bindings() {
    const OVERLAP: CardRules = CardRules {
        activated: &[activation(
            TargetPlan::Groups(&[
                TargetSpec::one(object(1, -1)),
                TargetSpec::one(object(1, -1)),
            ]),
            REWARD_PROGRAM,
        )],
        ..REWARD
    };
    const OBSERVER: CardRules = CardRules {
        effect: Effect::Creature(2, 1),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::CreatureDiesBatch {
                filter: CreatureFilter {
                    relation: -1,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        }],
        ..CardRules::EMPTY
    };
    let _scope =
        fixture_rules::activate(&[(Card::HillGiant, &OVERLAP), (Card::GoblinPiker, &OBSERVER)]);
    for overlap in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let other = if overlap {
            own
        } else {
            put(&mut g, p.other(), "Bear Cub", Zone::Battlefield)
        };
        reward(&mut g, source, Some(own), Some(other));
        drain(&mut g);
        assert_eq!(
            g.observe(p).creatures_died_this_turn,
            if overlap { 1 } else { 2 }
        );
        assert_eq!(
            g.state.players[p.index()].life,
            21,
            "one trigger for one destruction batch"
        );
        assert_eq!(
            g.counter_count(source, CounterKind::PlusOne),
            2,
            "one reward despite two group bindings"
        );
    }
}

#[test]
fn both_optional_role_groups_can_finish_without_targets_or_effects() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &REWARD)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    reward(&mut g, source, None, None);
    drain(&mut g);
    for id in [own, enemy] {
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    }
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
}
