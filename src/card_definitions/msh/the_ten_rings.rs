// Direct typed behavior from complete frozen Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::MaximumHandSize(10)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Threshold {
                value: StepValue::Hand,
                minimum: None,
                maximum: Some(9),
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DrawScaled(
            PlayerSet::Controller,
            AmountValue::ControllerHandDeficit(10),
        )],
    }],
    ..CardRules::EMPTY
};
