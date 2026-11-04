const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 7),
    keywords: Keywords::FLYING,
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::Threshold {
                value: StepValue::TotalCreatureToughness,
                minimum: Some(10),
                maximum: None,
            },
            event: &TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::If {
                guard: StepGuard::Threshold {
                    value: StepValue::TotalCreatureToughness,
                    minimum: Some(20),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::UntapPlayerCreatures(PlayerSet::Controller),
            Instruction::If {
                guard: StepGuard::Threshold {
                    value: StepValue::TotalCreatureToughness,
                    minimum: Some(40),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::LoseHalfLife(PlayerSet::Opponent),
        ],
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
