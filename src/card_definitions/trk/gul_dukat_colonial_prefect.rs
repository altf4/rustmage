// Oracle and official Star Trek release notes; pinned XMage class is absent.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::Any,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DrainLifeScaled(
            AmountValue::OpponentLifeLostThisTurn,
        )],
    }],
    ..CardRules::EMPTY
};
