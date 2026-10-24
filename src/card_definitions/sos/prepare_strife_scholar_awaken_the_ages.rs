// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/strife_scholar_awaken_the_ages"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
