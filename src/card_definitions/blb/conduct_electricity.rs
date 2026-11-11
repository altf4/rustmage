// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Creature),
            TargetSpec {
                rule: TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::Token,
                },
                min: 0,
                max: 1,
            },
        ]),
        &[Instruction::DamageByGroup { first: 6, other: 2 }],
    ),
    ..CardRules::EMPTY
};
