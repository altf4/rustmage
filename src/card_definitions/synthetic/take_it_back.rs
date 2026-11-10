pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Spell), &[Instruction::ReturnSpellToHand]),
    extras: &CardExtras {
        adventure_parent: Some(Card(4011)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
