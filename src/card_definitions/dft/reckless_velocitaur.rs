// Complete Oracle/XMage: the committed contribution event binds the exact Mount or Vehicle.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::YourMainPhase,
            event: &TriggerCondition::SelfCrewsOrSaddles,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Grow {
                subject: Subject::Entering,
                power: 2,
                toughness: 0,
            },
            Instruction::GrantKeywords {
                subject: Subject::Entering,
                keywords: Keywords::TRAMPLE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
