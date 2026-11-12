// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::FLYING,
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 4,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
