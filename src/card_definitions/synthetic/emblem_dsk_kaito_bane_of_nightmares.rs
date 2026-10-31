// Kaito's emblem: controlled Ninjas get +1/+1.
const BONUS: StaticModifier = StaticModifier {
    own_turn: false,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: Some("Ninja"),
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    stats: Stats {
        power: 1,
        toughness: 1,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::StaticModifier(0, BONUS)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Emblem,
    deck_limit: None,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_modifiers: &[BONUS],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
