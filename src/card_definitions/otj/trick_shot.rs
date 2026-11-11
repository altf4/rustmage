const TOKEN_CREATURE: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Creature,
    predicate: &TargetPredicate::Token,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Creature),
            TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TOKEN_CREATURE,
                    constraint: TargetGroupConstraint::OtherThanGroup(0),
                },
                min: 0,
                max: 1,
            },
        ]),
        &[Instruction::DamageByGroup { first: 6, other: 2 }],
    ),
    ..CardRules::EMPTY
};
