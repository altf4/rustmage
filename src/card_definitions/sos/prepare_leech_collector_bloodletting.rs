// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/leech_collector_bloodletting"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
