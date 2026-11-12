pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Creature),
            TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::Creature,
                    constraint: TargetGroupConstraint::OtherThanGroup(0),
                },
                min: 0,
                max: 1,
            },
        ]),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 2,
            },
            Instruction::SelectGroup(1),
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};
