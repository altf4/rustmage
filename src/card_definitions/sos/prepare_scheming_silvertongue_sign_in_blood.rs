// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/scheming_silvertongue_sign_in_blood"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
