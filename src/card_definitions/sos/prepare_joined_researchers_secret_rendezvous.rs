// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/joined_researchers_secret_rendezvous"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
