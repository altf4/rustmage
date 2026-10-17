// Direct typed behavior; complete pinned Oracle and XMage source reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::Threshold {
                value: StepValue::Creatures,
                minimum: Some(3),
                maximum: None,
            },
            event: &TriggerCondition::AtStep {
                step: TriggerStep::BeginCombat,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
