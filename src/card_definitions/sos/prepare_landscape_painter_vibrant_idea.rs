// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/landscape_painter_vibrant_idea"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
