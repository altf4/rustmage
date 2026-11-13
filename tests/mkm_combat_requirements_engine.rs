//! Rules requirements are scored together at each completed combat declaration.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CREATURE: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
const MUST_BLOCK: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURE,
        &[Instruction::RequireSubjectCombat {
            subject: Subject::Targets,
            requirement: CombatRequirement::Block,
            duration: CombatRequirementDuration::ThisTurn,
        }],
    ),
    ..CardRules::EMPTY
};
const MUST_ATTACK_OR_BLOCK: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURE,
        &[Instruction::RequireSubjectCombat {
            subject: Subject::Targets,
            requirement: CombatRequirement::AttackOrBlock,
            duration: CombatRequirementDuration::ThisTurn,
        }],
    ),
    ..CardRules::EMPTY
};
const MUST_BE_BLOCKED: CardRules = CardRules {
    effect: Effect::Spell(CREATURE, &[Instruction::RequireTargetsBlocked { group: 0 }]),
    ..CardRules::EMPTY
};
const MENACE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::MENACE,
    ..CardRules::EMPTY
};
const TAX: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::MENACE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::BlockTaxWhileAttacking(1)],
    }],
    ..CardRules::EMPTY
};
const BEAR_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::SourceDeclaredAttackThisCombat,
            event: &TriggerCondition::ControlledSubtypeAttacks {
                subtype: "Bear",
                exclude: false,
                one_or_more: false,
            },
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::RequireSubjectCombat {
            subject: Subject::Targets,
            requirement: CombatRequirement::BlockEventAttacker,
            duration: CombatRequirementDuration::ThisCombat,
        }],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn declare(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn block(blocker: CardId, attacker: CardId) -> Action {
    Action::Block { blocker, attacker }
}

fn cast_requirement(game: &mut Game, player: Player, name: &str, target: CardId) {
    let spell = put(game, player, name, Zone::Hand);
    cast(game, spell, Some(Target::Card(target)));
    settle(game);
}

#[test]
fn a_required_blocker_cannot_ignore_menace_or_be_forced_to_pay_a_tax() {
    let player = Player::P0;
    let defender = player.other();
    for (taxed, companion) in [(false, false), (false, true), (true, true)] {
        let _rules = fixture_rules::activate(&[
            (named("Opt"), &MUST_BLOCK),
            (named("Hill Giant"), if taxed { &TAX } else { &MENACE }),
        ]);
        let mut game = scenario(player);
        let attacker = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let required = put(&mut game, defender, "Grizzly Bears", Zone::Battlefield);
        let companion = companion.then(|| put(&mut game, defender, "Bear Cub", Zone::Battlefield));
        cast_requirement(&mut game, player, "Opt", required);
        declare(&mut game, &[attacker]);
        pass_pair(&mut game);
        assert_eq!(game.state.pending, Pending::Blockers);

        if !taxed && let Some(companion) = companion {
            reject(&mut game, Action::FinishBlockers);
            act(&mut game, block(required, attacker));
            reject(&mut game, Action::FinishBlockers);
            act(&mut game, block(companion, attacker));
            act(&mut game, Action::FinishBlockers);
            assert_eq!(
                game.state.blocks,
                [(required, attacker), (companion, attacker)]
            );
        } else {
            if companion.is_none() {
                reject(&mut game, block(required, attacker));
            }
            act(&mut game, Action::FinishBlockers);
            assert!(game.state.blocks.is_empty());
            assert!(game.observe(defender).block_payment.is_none());
        }
    }
}

fn resolve_attack_triggers(game: &mut Game, blocker: CardId) {
    for _ in 0..16 {
        match game.state.pending {
            Pending::Priority if game.state.stack.is_empty() => return,
            Pending::Priority => act(game, Action::Pass),
            Pending::OrderTriggers { .. } => {
                choose(game, |action| matches!(action, Action::PutTrigger { .. }));
            }
            Pending::Targets { .. } => act(
                game,
                Action::ChooseTarget {
                    target: Target::Card(blocker),
                },
            ),
            ref pending => panic!("unexpected combat trigger choice {pending:?}"),
        }
    }
    panic!("combat triggers did not finish");
}

#[test]
fn contradictory_pair_requirements_maximize_with_attacker_weights_and_menace() {
    let _rules = fixture_rules::activate(&[
        (named("Hill Giant"), &BEAR_OBSERVER),
        (named("Bear Cub"), &MENACE),
        (named("Shock"), &MUST_BE_BLOCKED),
    ]);
    let player = Player::P0;
    let defender = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let plain = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let menace = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let required = put(&mut game, defender, "Grizzly Bears", Zone::Battlefield);
    let companion = put(&mut game, defender, "Grizzly Bears", Zone::Battlefield);
    cast_requirement(&mut game, player, "Shock", menace);
    let mut source_stays_home = game.clone();
    declare(&mut source_stays_home, &[plain, menace]);
    assert!(source_stays_home.state.stack.is_empty());
    assert!(source_stays_home.state.pending_triggers.is_empty());

    declare(&mut game, &[source, plain, menace]);
    resolve_attack_triggers(&mut game, required);
    assert_eq!(
        game.state
            .combat_rules
            .as_ref()
            .unwrap()
            .creature_requirements
            .len(),
        2,
    );
    pass_pair(&mut game);
    assert_eq!(game.state.pending, Pending::Blockers);
    reject(&mut game, Action::FinishBlockers);
    // The two effects name the same blocker. It can satisfy either pair,
    // while putting both blockers on the menacing attacker also satisfies
    // the independent requirement for that attacker to be blocked.
    reject(&mut game, block(required, plain));
    reject(&mut game, block(companion, plain));
    let checkpoint = game.snapshot();
    let action = block(required, menace);
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, action);
    indexed.apply_index(defender, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    reject(&mut game, Action::FinishBlockers);
    act(&mut game, block(companion, menace));
    act(&mut game, Action::FinishBlockers);
    assert_eq!(game.state.blocks, [(required, menace), (companion, menace)]);

    indexed.restore(&checkpoint).unwrap();
    let extra = put(&mut indexed, defender, "Grizzly Bears", Zone::Battlefield);
    act(&mut indexed, block(required, plain));
    act(&mut indexed, block(companion, menace));
    act(&mut indexed, block(extra, menace));
    act(&mut indexed, Action::FinishBlockers);
    assert_eq!(
        indexed.state.blocks,
        [(required, plain), (companion, menace), (extra, menace)]
    );

    game.begin_step(Step::EndCombat);
    game.give_priority();
    pass_pair(&mut game);
    assert!(
        game.state
            .combat_rules
            .as_ref()
            .unwrap()
            .creature_requirements
            .is_empty()
    );
    assert!(!game.declared_attack_this_combat(game.object_ref(source)));
    assert_eq!(game.block_requirement_weight(menace), 1);
}

#[test]
fn an_attack_or_block_requirement_follows_control_and_lasts_through_each_combat() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &MUST_ATTACK_OR_BLOCK)]);
    let player = Player::P1;
    let defender = player.other();
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let other = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let exact = game.object_ref(creature);
    cast_requirement(&mut game, player, "Opt", creature);

    let mut stolen = game.clone();
    stolen.record_control(exact, defender, None);
    declare(&mut stolen, &[other]);
    pass_pair(&mut stolen);
    reject(&mut stolen, Action::FinishBlockers);
    act(&mut stolen, block(creature, other));
    act(&mut stolen, Action::FinishBlockers);

    let mut blinked = game.clone();
    blinked.move_object(creature, Zone::Exile);
    blinked.move_object(creature, Zone::Battlefield);
    blinked.record_control(blinked.object_ref(creature), defender, None);
    declare(&mut blinked, &[other]);
    pass_pair(&mut blinked);
    act(&mut blinked, Action::FinishBlockers);

    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    reject(&mut game, Action::FinishAttackers);
    declare(&mut game, &[creature]);
    game.add_combat(false);
    game.begin_step(Step::EndCombat);
    game.give_priority();
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::BeginCombat);
    game.state.objects[creature as usize].tapped = false;
    pass_pair(&mut game);
    assert_eq!(game.state.pending, Pending::Attackers);
    reject(&mut game, Action::FinishAttackers);
    declare(&mut game, &[creature]);

    game.begin_step(Step::End);
    game.give_priority();
    pass_pair(&mut game);
    assert!(!game.attacks_each_combat(creature));
}

#[test]
fn one_required_blocker_cannot_score_more_by_enabling_many_menace_bundles() {
    let _rules =
        fixture_rules::activate(&[(named("Opt"), &MUST_BLOCK), (named("Hill Giant"), &MENACE)]);
    let player = Player::P0;
    let defender = player.other();
    let mut game = scenario(player);
    let attackers: Vec<_> = (0..20)
        .map(|_| put(&mut game, player, "Hill Giant", Zone::Battlefield))
        .collect();
    let blockers: Vec<_> = (0..40)
        .map(|_| put(&mut game, defender, "Bear Cub", Zone::Battlefield))
        .collect();
    cast_requirement(&mut game, player, "Opt", blockers[0]);
    declare(&mut game, &attackers);
    pass_pair(&mut game);
    assert_eq!(game.state.pending, Pending::Blockers);
    assert_eq!(game.maximum_weighted_block_requirements(&[]), Some(1));
    let first = attackers[0];
    assert_eq!(
        game.maximum_weighted_block_requirements(&[(blockers[0], first), (blockers[1], first)]),
        Some(1)
    );
}
