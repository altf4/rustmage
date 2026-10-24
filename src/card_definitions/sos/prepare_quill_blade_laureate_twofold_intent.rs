// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/quill_blade_laureate_twofold_intent"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
