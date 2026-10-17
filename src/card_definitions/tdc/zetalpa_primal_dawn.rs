// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 8),
    triggers: &[],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::FLYING
        .union(Keywords::VIGILANCE)
        .union(Keywords::TRAMPLE)
        .union(Keywords::DOUBLE_STRIKE)
        .union(Keywords::INDESTRUCTIBLE),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::FLYING
                .union(Keywords::VIGILANCE)
                .union(Keywords::TRAMPLE)
                .union(Keywords::DOUBLE_STRIKE)
                .union(Keywords::INDESTRUCTIBLE),
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::FLYING
                    .union(Keywords::VIGILANCE)
                    .union(Keywords::TRAMPLE)
                    .union(Keywords::DOUBLE_STRIKE)
                    .union(Keywords::INDESTRUCTIBLE),
            )],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
