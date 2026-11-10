pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLASH.union(Keywords::FLYING),
    extras: &CardExtras {
        mayhem_cost: Some(ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
