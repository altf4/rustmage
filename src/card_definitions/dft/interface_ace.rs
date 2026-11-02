// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CrewSaddleWithToughness],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::YourTurn,
            event: &TriggerCondition::OncePerTurn(&TriggerCondition::SelfBecomesTapped),
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::UntapSource],
    }],
    ..CardRules::EMPTY
};
