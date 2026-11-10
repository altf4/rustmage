pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::OtherThanSource,
            },
            min: 0,
            max: 1,
        }),
        program: &[
            Instruction::RemoveAbilities {
                subject: Subject::Targets,
                duration: AnimationDuration::Permanent,
            },
            Instruction::ReplaceTargetCreatureForm(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Named(&["Lizard"]),
                base: Some((4, 4)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::Permanent,
            }),
            Instruction::SetColors {
                subject: Subject::Targets,
                colors: ColorValue::Fixed(16),
                duration: AnimationDuration::Permanent,
            },
        ],
    }],
    ..CardRules::EMPTY
};
