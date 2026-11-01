const BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::Count(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 0,
        exclude: 0,
        subtype: Some("Rat"),
        basic: false,
        other: true,
        cards_only: false,
    }),
    stats: Stats {
        power: 1,
        toughness: 1,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::ScalingModifier(0, BONUS)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    deck_limit: None,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        scaling_modifiers: &[BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
