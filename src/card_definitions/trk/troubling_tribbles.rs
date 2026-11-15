// Oracle and pinned XMage reviewed. Every token retains this upkeep trigger.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    keywords: Keywords::CANT_BLOCK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateCopyToken {
            subject: Subject::Source,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
