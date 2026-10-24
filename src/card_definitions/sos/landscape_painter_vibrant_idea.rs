// Direct typed behavior; complete Oracle and XMage obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::NONE,
    triggers: &[],
    extras: &CardExtras {
        prepared_spell: Some(Card(5371)),
        enters_prepared: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
