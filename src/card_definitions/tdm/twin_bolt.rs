// Directly authored from the complete pinned Oracle and XMage divided-damage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Any,
            min: 1,
            max: 2,
        }),
        &[Instruction::DamageDivided { total: 2 }],
    ),
    ..CardRules::EMPTY
};
