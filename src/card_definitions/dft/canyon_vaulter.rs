// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::YourMainPhase,
            event: &TriggerCondition::SelfCrewsOrSaddles,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantKeywords {
            subject: Subject::Entering,
            keywords: Keywords::FLYING,
        }],
    }],
    ..CardRules::EMPTY
};
