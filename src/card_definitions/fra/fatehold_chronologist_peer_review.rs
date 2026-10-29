// Directly authored from complete pinned Oracle and XMage; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    keywords: Keywords::FLYING,
    extras: &CardExtras {
        enters_prepared: true,
        prepared_spell: Some(Card(5444)),
        cost_alternatives: &[ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
