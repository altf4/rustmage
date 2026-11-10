// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    keywords: Keywords::HASTE,
    extras: &CardExtras {
        mayhem_cost: Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
