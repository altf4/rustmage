const PERMANENT_CARDS: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Graveyard,
    relation: 0,
    kind: 1 | 2 | 16 | 32 | 64 | 256,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: true,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Expression {
                    value: PERMANENT_CARDS,
                    minimum: Some(6),
                    maximum: None,
                },
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
