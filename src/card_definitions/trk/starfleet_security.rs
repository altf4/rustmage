// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::HASTE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            event: &TriggerCondition::SelfDies,
            guard: StepGuard::YourTurn,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5296),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
