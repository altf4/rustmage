// Core attack-requirement fixture. Whole card remains gated: Wall evasion pending.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttacksEachCombat],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AttacksEachCombat],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
