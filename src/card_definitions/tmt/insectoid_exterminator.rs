// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::PermanentLeftBattlefieldThisTurn,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Scry(1)],
    }],
    ..CardRules::EMPTY
};
