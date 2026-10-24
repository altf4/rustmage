// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/sanar_unfinished_genius_wild_idea"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
