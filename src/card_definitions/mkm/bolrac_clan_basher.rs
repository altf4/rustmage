const COMBAT_KEYWORDS: Keywords = Keywords::DOUBLE_STRIKE.union(Keywords::TRAMPLE);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: COMBAT_KEYWORDS,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::DOUBLE_STRIKE,
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::TRAMPLE)],
        },
    ],
    extras: &CardExtras {
        disguise_cost: Some(ManaCost {
            colored: [0, 0, 0, 2, 0, 0],
            generic: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
