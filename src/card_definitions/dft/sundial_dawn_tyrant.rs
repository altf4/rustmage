// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    extras: &CardExtras {
        has_rules_abilities: Some(false),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
