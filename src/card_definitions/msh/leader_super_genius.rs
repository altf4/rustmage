// Direct complete Oracle behavior using event-local connive replacements.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(
            0,
            ReplacementSpec::ConniveDraw,
        )],
    }],
    extras: &CardExtras {
        replacement_effects: &[ReplacementSpec::ConniveDraw],
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[Instruction::Connive {
            subject: Subject::Targets,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
