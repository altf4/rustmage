// Pinned Oracle and XMage behavior; gated pending integration and rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::Any,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5454),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
