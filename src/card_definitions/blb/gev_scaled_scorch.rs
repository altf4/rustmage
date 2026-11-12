pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::OtherControlledCreatureEntryCountersByStat {
                kind: CounterKind::PlusOne,
                amount: StatValue::OpponentLostLifeThisTurn,
            },
        ],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPayCosts {
                subject: CounterSubject::Related,
                costs: &[ResolutionCost {
                    life: Some(AmountValue::Fixed(2)),
                    ..ResolutionCost::FREE
                }],
                exile: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Subtype("Lizard"),
                ordinal: None,
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            })),
            program: &[Instruction::Damage(1)],
        },
    ],
    ..CardRules::EMPTY
};
