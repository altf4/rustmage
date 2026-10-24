// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/scathing_shadelock_venomous_words"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
