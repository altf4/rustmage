// Direct typed behavior; complete Oracle and XMage obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLYING,
    triggers: &[],
    extras: &CardExtras {
        prepared_spell: Some(Card(5381)),
        enters_prepared: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
