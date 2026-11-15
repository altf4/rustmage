// Complete Oracle behavior, checked against the official Star Trek release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FIRST_STRIKE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            event: &TriggerCondition::SelfBecomesTapped,
            guard: StepGuard::EquipmentOrSpacecraftEnteredThisTurn,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
