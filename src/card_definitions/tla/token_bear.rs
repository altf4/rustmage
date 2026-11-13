pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    deck_limit: None,
    extras: &CardExtras {
        has_rules_abilities: Some(false),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
