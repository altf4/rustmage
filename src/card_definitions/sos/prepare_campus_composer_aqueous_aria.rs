// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/campus_composer_aqueous_aria"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
