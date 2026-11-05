// Whole Oracle: any creature returns to its owner; Harmonize uses the same target.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Creature), &[Instruction::ReturnToHand]),
    extras: &CardExtras {
        harmonize_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 5,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
