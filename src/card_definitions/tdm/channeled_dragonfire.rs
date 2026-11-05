// Directly authored from the pinned Oracle and complete XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        &[Instruction::Damage(2)],
    ),
    extras: &CardExtras {
        harmonize_cost: Some(ManaCost {
            colored: [0, 0, 0, 2, 0, 0],
            generic: 5,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
