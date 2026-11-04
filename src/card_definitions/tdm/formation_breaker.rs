const BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::AtLeast {
        value: &StatValue::CounterBearingCreatures,
        minimum: 1,
    },
    stats: Stats {
        power: 1,
        toughness: 2,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CannotBeBlockedByCreaturesWithLessPower],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ScalingModifier(0, BONUS)],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        scaling_modifiers: &[BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
