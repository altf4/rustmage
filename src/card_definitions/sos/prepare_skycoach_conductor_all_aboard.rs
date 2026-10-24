// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/skycoach_conductor_all_aboard"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
