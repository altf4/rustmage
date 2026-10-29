pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Emblem,
    deck_limit: None,
    static_groups: &[StaticAbilityGroup { line: 0, first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::StaticModifier(0, StaticModifier {
            own_turn: false, requires_life_gain: false,
            filter: CreatureFilter { relation: 0, other: false, subtype: None, keyword: Keywords::NONE,
                attacking: false, tokens: None, artifact: false },
            stats: Stats { power: 2, toughness: 2 },
        })] }],
    extras: &CardExtras { static_modifiers: &[StaticModifier {
        own_turn: false, requires_life_gain: false,
        filter: CreatureFilter { relation: 0, other: false, subtype: None, keyword: Keywords::NONE,
            attacking: false, tokens: None, artifact: false },
        stats: Stats { power: 2, toughness: 2 },
    }], ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
