// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledKindOrSubtypeTapMana {
            kinds: 2,
            subtype: "Ally",
            production: ManaProduction::any_color(1),
        }],
    }],
    ..CardRules::EMPTY
};
