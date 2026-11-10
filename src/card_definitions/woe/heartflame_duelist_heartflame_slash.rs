// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledInstantSorceryKeywords(
            Keywords::LIFELINK,
        )],
    }],
    extras: &CardExtras {
        adventure_spell: Some(Card(5554)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
