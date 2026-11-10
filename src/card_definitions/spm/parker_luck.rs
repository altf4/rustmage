pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Player {
                opponent_only: false,
            },
            min: 2,
            max: 2,
        }),
        program: &[Instruction::RevealTargetTopsToHandLoseOtherManaValue],
    }],
    ..CardRules::EMPTY
};
