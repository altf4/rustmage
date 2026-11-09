pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::AlternateRevealedLibraryPicks(5)],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 7,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
