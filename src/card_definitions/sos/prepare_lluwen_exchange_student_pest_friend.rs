// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/lluwen_exchange_student_pest_friend"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
