// Direct typed behavior from the complete Oracle body and pinned XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::Attacked,
            event: &TriggerCondition::SelfEnters,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopChoosePlay {
            amount: 2,
            next_turn: true,
        }],
    }],
    ..CardRules::EMPTY
};
