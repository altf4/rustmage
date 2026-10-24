// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/studious_first_year_rampant_growth"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
