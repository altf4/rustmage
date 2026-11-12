pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::SourceUnsolved(&StepGuard::Threshold {
                value: StepValue::Lands,
                minimum: Some(7),
                maximum: None,
            }),
            event: &TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::SolveSource],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AdditionalLandPlays(1)],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[
                StaticAbilityFragment::LookAtLibraryTop,
                StaticAbilityFragment::PlayLandFromLibraryTop,
                StaticAbilityFragment::CastCreatureFromLibraryTop,
                StaticAbilityFragment::CastFromLibraryTop(CastFilter::Enchantment),
            ],
        },
    ],
    extras: &CardExtras {
        solved_static_groups: &[2],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
