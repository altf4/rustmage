// Direct typed behavior from the complete pinned Oracle and XMage sources.
const MAX_SPEED: StatValue = StatValue::AtLeast {
    value: &StatValue::ControllerSpeed,
    minimum: 4,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::StartYourEngines],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::NoMaximumHandSize],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::ConditionalDoubleDraw(MAX_SPEED)],
        },
    ],
    ..CardRules::EMPTY
};
