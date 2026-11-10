// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    extras: &CardExtras {
        mayhem_cost: Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
