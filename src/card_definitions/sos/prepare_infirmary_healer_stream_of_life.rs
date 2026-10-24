// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Unsupported,
    extras: &CardExtras {
        prepared_identity: Some("@prepare/infirmary_healer_stream_of_life"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
