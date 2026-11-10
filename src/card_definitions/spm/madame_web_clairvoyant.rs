pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::LookAtLibraryTop],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CastFromLibraryTop(
                CastFilter::AnyOf(&[CastFilter::Subtype("Spider"), CastFilter::Noncreature]),
            )],
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackDeclared {
            minimum: 1,
            power_min: None,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayMill(1),
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(1)),
        ],
    }],
    ..CardRules::EMPTY
};
