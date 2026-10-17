// Direct typed behavior; complete Oracle and pinned XMage reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    keywords: Keywords::NONE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            event: &TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::Any,
                guard: StepGuard::Always,
            },
            guard: StepGuard::CreatureDied,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::UntapSource],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SelfUntap(
            UntapRestriction::ControllerStep,
        )],
    }],
    ..CardRules::EMPTY
};
