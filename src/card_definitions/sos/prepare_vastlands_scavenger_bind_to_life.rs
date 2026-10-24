// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/vastlands_scavenger_bind_to_life"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
