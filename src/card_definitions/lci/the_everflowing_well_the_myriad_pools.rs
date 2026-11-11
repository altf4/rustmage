const DESCEND_EIGHT: StepGuard = StepGuard::Expression {
    value: StatValue::Count(CountFilter {
        zone: CountZone::Graveyard,
        relation: 0,
        kind: 371,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: true,
    }),
    minimum: Some(8),
    maximum: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
                Instruction::Draw(PlayerSet::Controller, 2),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Upkeep,
                relation: CastRelation::You,
                guard: DESCEND_EIGHT,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::TransformSource],
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
