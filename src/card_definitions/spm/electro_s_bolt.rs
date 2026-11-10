pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Creature), &[Instruction::Damage(4)]),
    extras: &CardExtras {
        mayhem_cost: Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
