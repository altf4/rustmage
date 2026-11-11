const ANY_PERMANENT: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
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
        generic: 1,
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
            condition: TriggerCondition::OncePerTurn(&TriggerCondition::CreatureDiesBatch {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
            }),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5246),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::ClassLevelAtLeast(2),
                event: &TriggerCondition::Sacrificed {
                    filter: ANY_PERMANENT,
                    own_turn: false,
                },
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            })),
            program: &[Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(2))],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::ClassLevelAtLeast(3),
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::End,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayCosts {
                    costs: &[ResolutionCost {
                        objects: Some(ResolutionObjectCost {
                            cost: AdditionalCost {
                                kind: EffectChoiceKind::Sacrifice,
                                filter: SacrificeFilter {
                                    nonland: true,
                                    other: true,
                                    ..ANY_PERMANENT
                                },
                            },
                            count: 3,
                        }),
                        ..ResolutionCost::FREE
                    }],
                    skip: 1,
                },
                Instruction::ChooseGraveyardReturnWithCounter(CounterKind::Finality),
            ],
        },
    ],
    activated: &[
        LEVEL_TWO,
        ActivatedAbility {
            guard: ActivationGuard::ClassLevelExactly(2),
            cost: ActivationCost {
                generic: 2,
                ..LEVEL_TWO.cost
            },
            program: &[Instruction::GainClassLevel(3)],
            ..LEVEL_TWO
        },
    ],
    ..CardRules::EMPTY
};
