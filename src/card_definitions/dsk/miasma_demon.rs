pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::MayDiscardAnyReflexive(1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: usize::MAX,
            }),
            program: &[
                Instruction::UpToTargets(AmountValue::ReflexiveTargetLimit),
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: -2,
                    toughness: -2,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
