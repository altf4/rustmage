// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::MENACE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::MENACE)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::Threshold {
                value: StepValue::GreatestCreaturePower,
                minimum: Some(4),
                maximum: None,
            },
            event: &TriggerCondition::SelfAttacks,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow {
            subject: Subject::Source,
            power: 2,
            toughness: 2,
        }],
    }],
    ..CardRules::EMPTY
};
