// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    extras: &CardExtras {
        web_slinging_cost: Some(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
