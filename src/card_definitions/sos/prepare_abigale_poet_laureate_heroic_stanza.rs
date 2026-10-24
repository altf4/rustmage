// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/abigale_poet_laureate_heroic_stanza"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
