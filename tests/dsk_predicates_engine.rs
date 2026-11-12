//! Target qualifications and effective signed power in live and projected worlds.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const UNTAPPED_CREATURES: TargetRule = TargetRule::Filtered {
    base: &TargetRule::creature_you_control(),
    predicate: &TargetPredicate::Untapped,
};
const OPPOSING_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const TAP_AND_HIT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec {
                rule: UNTAPPED_CREATURES,
                min: 1,
                max: 2,
            },
            TargetSpec::one(OPPOSING_CREATURE),
        ]),
        &[
            Instruction::SelectGroup(0),
            Instruction::Tap,
            Instruction::GroupPowerDamage {
                source: 0,
                recipient: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};
const TAP_ONE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::Tap],
    ),
    ..CardRules::EMPTY
};

const HAND_SCALING: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::ControllerHandSize,
    stats: Stats {
        power: 1,
        toughness: 1,
    },
};
const HAND_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::ScalingModifier(0, HAND_SCALING)],
}];
const HAND_SIZED_CREATURE: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    static_groups: HAND_GROUPS,
    extras: &CardExtras {
        static_ability_groups: HAND_GROUPS,
        scaling_modifiers: &[HAND_SCALING],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const PROJECTED_REDUCTION: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Fixed(2),
    turn: CostTurn::Any,
    guard: StepGuard::Threshold {
        value: StepValue::ControlledCreaturePowerAtMost {
            maximum: 5,
            other: false,
        },
        minimum: Some(1),
        maximum: None,
    },
};
const REDUCTION_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CostReduction(0, PROJECTED_REDUCTION)],
}];
const EXILE_SMALL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::ToughnessAtMost(5),
        })),
        &[Instruction::Exile],
    ),
    static_groups: REDUCTION_GROUPS,
    extras: &CardExtras {
        static_ability_groups: REDUCTION_GROUPS,
        cost_reductions: &[PROJECTED_REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const DRAW_TWO: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 2)],
    ),
    ..CardRules::EMPTY
};

const POWER_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(-1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::RememberCount(AmountValue::ControlledCreaturePowerAtMost {
                maximum: 2,
                other: false,
            }),
            Instruction::DamagePlayersScaled(PlayerSet::Opponent, AmountValue::RememberedCount),
            Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::RememberedCount),
            Instruction::If {
                guard: StepGuard::Threshold {
                    value: StepValue::ControlledCreaturePowerAtMost {
                        maximum: -1,
                        other: true,
                    },
                    minimum: Some(1),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
const WEAKEN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::Grow {
            subject: Subject::Targets,
            power: -3,
            toughness: 0,
        }],
    ),
    ..CardRules::EMPTY
};

fn choose_target(target: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(target),
    }
}

#[test]
fn untapped_targets_are_rechecked_before_a_group_is_tapped_and_deals_damage() {
    let growth = Card::from_name("Giant Growth").unwrap();
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(growth, &TAP_AND_HIT), (opt, &TAP_ONE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let tapped = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[tapped as usize].tapped = true;
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let opponent = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let hit = spawn(&mut game, player, growth, Zone::Hand);
    let tap = spawn(&mut game, player.other(), opt, Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: hit });
    reject(&mut game, choose_target(tapped));
    act(&mut game, choose_target(first));
    act(&mut game, choose_target(second));
    act(&mut game, choose_target(opponent));
    pay(&mut game);

    act(&mut game, Action::Pass);
    cast(&mut game, tap, Some(Target::Card(first)));
    settle(&mut game);
    assert!(game.object_view(first).tapped);
    assert!(game.object_view(second).tapped);
    assert!(!game.object_view(opponent).tapped);
    assert_eq!(game.object_view(opponent).damage, 2);
}

#[test]
fn toughness_target_and_power_discount_see_spell_departure_and_recheck_live_stats() {
    let drake = Card::from_name("Enigma Drake").unwrap();
    let study = Card::from_name("Quick Study").unwrap();
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[
        (drake, &HAND_SIZED_CREATURE),
        (study, &EXILE_SMALL),
        (opt, &DRAW_TWO),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let creature = spawn(&mut game, player, drake, Zone::Battlefield);
    let exile = spawn(&mut game, player, study, Zone::Hand);
    let response = spawn(&mut game, player, opt, Zone::Hand);
    for _ in 0..4 {
        put(&mut game, player, "Forest", Zone::Hand);
    }
    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 0];
    assert_eq!(stats(&game, creature), (6, 6));
    // Only the projected departure makes both the target and cost legal.
    cast(&mut game, exile, Some(Target::Card(creature)));
    assert_eq!(stats(&game, creature), (5, 5));
    assert_eq!(
        game.observe(player).players[player.index()].mana,
        [0, 1, 0, 0, 0, 0]
    );

    let mut unchanged = game.clone();
    settle(&mut unchanged);
    assert_eq!(unchanged.object_view(creature).zone, Zone::Exile);

    cast(&mut game, response, None);
    settle(&mut game);
    assert_eq!(stats(&game, creature), (6, 6));
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(game.object_view(exile).zone, Zone::Graveyard);
}

#[test]
fn power_count_samples_live_signed_power_and_can_exclude_the_exact_source() {
    let hill_giant = Card::from_name("Hill Giant").unwrap();
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(hill_giant, &POWER_OBSERVER), (opt, &WEAKEN)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let observer = spawn(&mut game, player, hill_giant, Zone::Hand);
    let raised = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let weakened = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    spawn(&mut game, player.other(), hill_giant, Zone::Battlefield);
    put(&mut game, player, "Forest", Zone::Battlefield);
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    let reduction = spawn(&mut game, player, opt, Zone::Hand);
    cast(&mut game, observer, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.observe(player).stack.len(), 1);

    let mut unchanged = game.clone();
    settle(&mut unchanged);
    let before = unchanged.observe(player);
    assert_eq!(before.players[player.index()].life, 23);
    assert_eq!(before.players[player.other().index()].life, 17);
    assert_eq!(before.players[player.index()].cards_drawn_this_turn, 0);

    cast(&mut game, growth, Some(Target::Card(raised)));
    cast(&mut game, reduction, Some(Target::Card(weakened)));
    settle(&mut game);
    assert_eq!(stats(&game, raised).0, 5);
    assert_eq!(stats(&game, weakened).0, -1);
    let after = game.observe(player);
    assert_eq!(after.players[player.index()].life, 22);
    assert_eq!(after.players[player.other().index()].life, 18);
    assert_eq!(after.players[player.index()].cards_drawn_this_turn, 1);
}
