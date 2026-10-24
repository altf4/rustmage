// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/kirol_history_buff_pack_a_punch"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
