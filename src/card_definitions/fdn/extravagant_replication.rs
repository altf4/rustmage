pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec::one(
            TargetRule::another_nonland_permanent_you_control(),
        )),
        program: &[Instruction::CreateCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
