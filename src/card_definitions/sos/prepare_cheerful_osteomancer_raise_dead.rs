// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/cheerful_osteomancer_raise_dead"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
