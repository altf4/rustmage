// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    keywords: Keywords::CHANGELING,
    extras: &CardExtras {
        has_convoke: true,
        as_enters_copy_creature: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
