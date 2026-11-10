//! Edge of Eternities combat restrictions, block events and prevention.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const DRONE: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLYING,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CanBlockOnlyFlying],
    }],
    ..CardRules::EMPTY
};
const STINGER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::REACH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBlocksCreatureWithKeyword(Keywords::FLYING),
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow {
            subject: Subject::Source,
            power: 5,
            toughness: 0,
        }],
    }],
    ..CardRules::EMPTY
};
const BALOTH: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CombatDamageCantBePrevented],
    }],
    ..CardRules::EMPTY
};
const PROTECTED: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
    keywords: Keywords::PROTECTION_RED,
    ..CardRules::EMPTY
};

fn pass_until(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..16 {
        if ready(game) {
            return;
        }
        if let Some(range) = game.damage_assignment_range() {
            act(
                game,
                Action::AssignDamage {
                    target: range.target,
                    amount: range.maximum,
                },
            );
        } else {
            act(game, Action::Pass);
        }
    }
    panic!("combat did not reach the requested step");
}

fn attack(game: &mut Game, creatures: &[CardId]) {
    pass_until(game, |g| g.state.pending == Pending::Attackers);
    for &card in creatures {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
    pass_until(game, |g| g.state.pending == Pending::Blockers);
}

fn blank(game: &mut Game, card: CardId) {
    let reference = game.object_ref(card);
    let controller = game.object_view(card).controller;
    game.remove_abilities_exact(
        &[reference],
        reference,
        controller,
        AnimationDuration::EndOfTurn,
    );
}

#[test]
fn flying_only_blocker_checks_the_attackers_actual_flying_ability() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &DRONE)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let drone = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    let flyer = put(&mut game, p, "Healer's Hawk", Zone::Battlefield);
    let ground = put(&mut game, p, "Magnigoth Sentry", Zone::Battlefield);
    attack(&mut game, &[flyer, ground]);
    let block_ground = Action::Block {
        blocker: drone,
        attacker: ground,
    };
    reject(&mut game, block_ground.clone());
    let mut no_flying = game.clone();
    blank(&mut no_flying, flyer);
    reject(
        &mut no_flying,
        Action::Block {
            blocker: drone,
            attacker: flyer,
        },
    );
    let mut no_restriction = game.clone();
    blank(&mut no_restriction, drone);
    act(&mut no_restriction, block_ground);
    act(&mut no_restriction, Action::FinishBlockers);
    act(
        &mut game,
        Action::Block {
            blocker: drone,
            attacker: flyer,
        },
    );
    act(&mut game, Action::FinishBlockers);
    assert_eq!(game.state.blocks, vec![(drone, flyer)]);
    assert_eq!(no_restriction.state.blocks, vec![(drone, ground)]);
}

#[test]
fn a_qualified_block_queues_once_and_survives_later_loss_of_flying() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &STINGER)]);
    let p = Player::P1;
    let mut game = scenario(p);
    let stinger = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let flyer = put(&mut game, p, "Healer's Hawk", Zone::Battlefield);
    let ground = put(&mut game, p, "Magnigoth Sentry", Zone::Battlefield);
    attack(&mut game, &[flyer, ground]);

    let mut ordinary = game.clone();
    act(
        &mut ordinary,
        Action::Block {
            blocker: stinger,
            attacker: ground,
        },
    );
    act(&mut ordinary, Action::FinishBlockers);
    assert!(ordinary.state.stack.is_empty());
    assert_eq!(stats(&ordinary, stinger), (3, 3));

    let action = Action::Block {
        blocker: stinger,
        attacker: flyer,
    };
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    game.apply_index(p.other(), game.revision(), index).unwrap();
    assert!(game.state.stack.is_empty());
    act(&mut game, Action::FinishBlockers);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.state.stack[0].source.unwrap().id, stinger);
    blank(&mut game, flyer);
    assert!(!game.keywords(flyer).contains(Keywords::FLYING));
    settle(&mut game);
    assert_eq!(stats(&game, stinger), (8, 3));
}

#[test]
fn combat_only_prohibition_preserves_shield_removal_and_noncombat_prevention() {
    let _rules =
        fixture_rules::activate(&[(Card::HillGiant, &BALOTH), (Card::GrizzlyBears, &PROTECTED)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let attacker = put(&mut game, p, "Goblin Piker", Zone::Battlefield);
    let blocker = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let mountain = put(&mut game, p, "Mountain", Zone::Battlefield);
    let forest = put(&mut game, p, "Forest", Zone::Battlefield);
    let sweeper = put(&mut game, p, "Pyroclasm", Zone::Hand);
    game.add_counter(blocker, CounterKind::Shield, 1);
    attack(&mut game, &[attacker]);
    act(&mut game, Action::Block { blocker, attacker });
    act(&mut game, Action::FinishBlockers);
    pass_until(&mut game, |g| {
        g.state.step == Step::CombatDamage && g.state.pending == Pending::Priority
    });
    assert_eq!(game.counter_count(blocker, CounterKind::Shield), 0);
    assert_eq!(game.object_view(blocker).damage, 2);
    assert_eq!(game.object_view(blocker).zone, Zone::Battlefield);

    pass_until(&mut game, |g| g.state.step == Step::Main2);
    act(&mut game, Action::TapMana { card: mountain });
    act(&mut game, Action::TapMana { card: forest });
    cast(&mut game, sweeper, None);
    settle(&mut game);
    assert_eq!(game.object_view(attacker).zone, Zone::Graveyard);
    assert_eq!(game.object_view(blocker).zone, Zone::Battlefield);
    assert_eq!(game.object_view(blocker).damage, 2);
}
