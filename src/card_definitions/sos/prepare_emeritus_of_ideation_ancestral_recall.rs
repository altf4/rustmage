// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Unsupported,
    extras: &CardExtras {
        prepared_identity: Some("@prepare/emeritus_of_ideation_ancestral_recall"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
