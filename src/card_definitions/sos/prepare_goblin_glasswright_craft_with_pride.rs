// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/goblin_glasswright_craft_with_pride"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
