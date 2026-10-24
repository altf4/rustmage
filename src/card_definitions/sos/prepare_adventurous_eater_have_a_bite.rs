// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/adventurous_eater_have_a_bite"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
