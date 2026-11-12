//! Completed draws, temporary abilities, and the empty-library replacement.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const LIBRARY_EMPTY: StatValue = StatValue::ControllerLibraryEmpty;
const EMPTY_BASE: BaseStatSetter = BaseStatSetter {
    power: Some(10),
    toughness: Some(10),
    own_turn: false,
    condition: Some(LIBRARY_EMPTY),
};
const EMPTY_KEYWORDS: Keywords = Keywords::FLYING.union(Keywords::VIGILANCE);
const CONUNDRUM: CardRules = CardRules {
    effect: Effect::Creature(2, 5),
    keywords: Keywords::HEXPROOF,
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::SkipEmptyLibraryDraws],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::SourceConditionalKeywords {
                    condition: LIBRARY_EMPTY,
                    keywords: EMPTY_KEYWORDS,
                },
                StaticAbilityFragment::BaseStatSetter(0, EMPTY_BASE),
            ],
        },
    ],
    extras: &CardExtras {
        base_stat_setter: Some(EMPTY_BASE),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const DRAW_THREE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 3)],
    ),
    ..CardRules::EMPTY
};
const DRAW_AND_BOUNCE: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[
            Instruction::Draw(PlayerSet::Controller, 3),
            Instruction::ReturnToHand,
        ],
    ),
    ..CardRules::EMPTY
};
const DRAW_ONE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};

const ANALYST: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::DEFENDER,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardDrawn {
            relation: CastRelation::You,
            ordinal: Some(2),
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::RemoveKeywords {
                subject: Subject::Source,
                keywords: Keywords::DEFENDER,
            },
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::VIGILANCE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
const MEMORY: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::Expression {
                value: StatValue::CardsDrawnThisTurn,
                minimum: Some(2),
                maximum: None,
            },
            event: &TriggerCondition::AtStep {
                step: TriggerStep::BeginCombat,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::PutCountersScaled {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: AmountValue::Stat(StatValue::CardsDrawnAfterFirstThisTurn),
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

#[test]
fn empty_draws_are_replaced_only_for_the_live_controller() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &CONUNDRUM),
        (named("Quick Study"), &DRAW_THREE),
        (named("Opt"), &DRAW_AND_BOUNCE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let draw = put(&mut game, player, "Quick Study", Zone::Hand);
    let draw_and_bounce = put(&mut game, player, "Opt", Zone::Hand);
    let library = game.state.players[player.index()].library.clone();
    for &card in &library[..library.len() - 1] {
        game.move_object(card, Zone::Exile);
    }
    assert_eq!(stats(&game, source), (2, 5));
    assert!(!game.object_view(source).keywords.intersects(EMPTY_KEYWORDS));

    let mut left_during_resolution = game.clone();
    cast(
        &mut left_during_resolution,
        draw_and_bounce,
        Some(Target::Card(source)),
    );
    settle(&mut left_during_resolution);
    assert_eq!(left_during_resolution.object_view(source).zone, Zone::Hand);
    assert_eq!(left_during_resolution.outcome(), None);
    assert_eq!(
        left_during_resolution.state.players[player.index()].cards_drawn_this_turn,
        1
    );
    cast(&mut left_during_resolution, draw, None);
    settle(&mut left_during_resolution);
    assert_eq!(
        left_during_resolution.outcome(),
        Some(Outcome::Winner(player.other()))
    );

    let mut stolen = game.clone();
    let reference = stolen.object_ref(source);
    stolen.record_control(reference, player.other(), None);
    cast(&mut stolen, draw, None);
    settle(&mut stolen);
    assert_eq!(stolen.outcome(), Some(Outcome::Winner(player.other())));

    cast(&mut game, draw, None);
    settle(&mut game);
    assert_eq!(game.outcome(), None);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(stats(&game, source), (10, 10));
    assert!(game.object_view(source).keywords.contains(EMPTY_KEYWORDS));
    game.add_counter(source, CounterKind::PlusOne, 1);
    assert_eq!(stats(&game, source), (11, 11));
    game.move_object(library[0], Zone::Library);
    assert_eq!(stats(&game, source), (3, 6));
    assert!(!game.object_view(source).keywords.intersects(EMPTY_KEYWORDS));
    assert!(
        game.object_view(source)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
}

#[test]
fn the_second_draw_changes_abilities_and_combat_counts_completed_draws_live() {
    let memory = named("Authority of the Consuls");
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &ANALYST),
        (memory, &MEMORY),
        (named("Opt"), &DRAW_ONE),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let analyst = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let first = put(&mut game, player, "Opt", Zone::Hand);
    let second = put(&mut game, player, "Opt", Zone::Hand);
    let third = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, first, None);
    pass_pair(&mut game);
    assert!(game.state.stack.is_empty());
    assert!(
        game.object_view(analyst)
            .keywords
            .contains(Keywords::DEFENDER)
    );

    cast(&mut game, second, None);
    pass_pair(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    let keywords = game.object_view(analyst).keywords;
    assert!(!keywords.contains(Keywords::DEFENDER));
    assert!(keywords.contains(Keywords::VIGILANCE));

    spawn(&mut game, player, memory, Zone::Battlefield);
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(analyst),
        },
    );
    cast(&mut game, third, None);
    pass_pair(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.counter_count(analyst, CounterKind::PlusOne), 2);

    game.begin_step(Step::End);
    pass_pair(&mut game);
    let keywords = game.object_view(analyst).keywords;
    assert!(keywords.contains(Keywords::DEFENDER));
    assert!(!keywords.contains(Keywords::VIGILANCE));
}
