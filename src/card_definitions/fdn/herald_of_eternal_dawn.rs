// Complete pinned Oracle/XMage body, directly authored typed behavior.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::FLASH.union(Keywords::FLYING),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CantLoseGameAndOpponentsCantWin],
        },
    ],
    ..CardRules::EMPTY
};
