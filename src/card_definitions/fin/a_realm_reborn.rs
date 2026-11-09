const MANA_GRANT: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::OtherControlledPermanentTapMana(
        ManaProduction::any_color(1),
    )],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: MANA_GRANT,
    extras: &CardExtras {
        static_ability_groups: MANA_GRANT,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
