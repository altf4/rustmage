// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/tam_observant_sequencer_deep_sight"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
