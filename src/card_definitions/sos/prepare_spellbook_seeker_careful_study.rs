// Preparation spell characteristics, gated until every printed obligation is authored.
pub(in crate::cards) const RULES: CardRules = CardRules {
    extras: &CardExtras {
        prepared_identity: Some("@prepare/spellbook_seeker_careful_study"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
