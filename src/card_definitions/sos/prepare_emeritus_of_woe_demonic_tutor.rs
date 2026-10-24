// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/emeritus_of_woe_demonic_tutor"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
