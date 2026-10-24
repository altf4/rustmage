// Direct typed behavior; complete Oracle and XMage obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FIRST_STRIKE,
    triggers: &[],
    extras: &CardExtras {
        prepared_spell: Some(Card(5366)),
        enters_prepared: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
