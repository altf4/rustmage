const SMALL_HAND: StepGuard = StepGuard::Expression {
    value: StatValue::OpponentHandSize,
    minimum: None,
    maximum: Some(1),
};
const NONLAND: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: true,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const LEVEL_TWO: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::ClassLevelExactly(1),
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        colored: [0, 0, 1, 0, 0, 0],
        mana_component: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::GainClassLevel(2)],
    sorcery: true,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Discard,
                    players: PlayerSet::Opponent,
                    filter: NONLAND,
                    amount: 1,
                    optional_skip: Some(1),
                },
                Instruction::Skip(1),
                Instruction::Discard(PlayerSet::Opponent, 2),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::ClassLevelAtLeast(2),
                event: &TriggerCondition::InterveningIf {
                    guard: SMALL_HAND,
                    event: &TriggerCondition::AtStep {
                        step: TriggerStep::Upkeep,
                        relation: CastRelation::Opponent,
                        guard: StepGuard::Always,
                    },
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::LoseLife(PlayerSet::Opponent, 2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::ClassLevelAtLeast(3),
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::Draw,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::If {
                    guard: SMALL_HAND,
                    skip: 1,
                },
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
    ],
    activated: &[
        LEVEL_TWO,
        ActivatedAbility {
            guard: ActivationGuard::ClassLevelExactly(2),
            cost: ActivationCost {
                generic: 3,
                ..LEVEL_TWO.cost
            },
            program: &[Instruction::GainClassLevel(3)],
            ..LEVEL_TWO
        },
    ],
    ..CardRules::EMPTY
};
