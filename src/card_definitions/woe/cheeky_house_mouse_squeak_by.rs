// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    extras: &CardExtras {
        has_rules_abilities: Some(false),
        adventure_spell: Some(Card(5548)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
