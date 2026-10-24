// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/honorbound_page_forum_s_favor"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
