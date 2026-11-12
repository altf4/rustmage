const BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::AtLeast {
        value: &StatValue::UnlockedRoomDoors,
        minimum: 2,
    },
    stats: Stats {
        power: 3,
        toughness: 0,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::ScalingModifier(0, BONUS)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    static_groups: GROUPS,
    extras: &CardExtras {
        scaling_modifiers: &[BONUS],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
