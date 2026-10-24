// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/maelstrom_artisan_rocket_volley"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
