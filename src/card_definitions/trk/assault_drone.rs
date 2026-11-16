// Pinned Oracle and XMage: live power bonus to controlled artifact creatures.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ControlledCreatureKindBonus {
            kinds: 17,
            stats: Stats {
                power: 1,
                toughness: 0,
            },
        }],
    }],
    ..CardRules::EMPTY
};
