// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Unsupported,
    extras: &CardExtras {
        prepared_identity: Some("@prepare/jadzi_steward_of_fate_oracle_s_gift"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
