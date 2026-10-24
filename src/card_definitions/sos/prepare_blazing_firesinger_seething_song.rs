// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/blazing_firesinger_seething_song"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
