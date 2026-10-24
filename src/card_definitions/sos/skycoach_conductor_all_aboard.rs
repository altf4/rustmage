// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::FLASH
        .union(Keywords::FLYING)
        .union(Keywords::VIGILANCE),
    extras: &CardExtras {
        prepared_spell: Some(Card(5380)),
        enters_prepared: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
