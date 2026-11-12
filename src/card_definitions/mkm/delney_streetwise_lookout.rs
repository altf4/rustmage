// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[
                StaticAbilityFragment::ControlledCreaturesCannotBeBlockedByPower {
                    attacker_maximum: 2,
                    blocker_minimum: 3,
                },
            ],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AdditionalTriggers {
                scope: AdditionalTriggerScope::ControlledCreaturePowerAtMost(2),
                condition: StatValue::Fixed(1),
            }],
        },
    ],
    ..CardRules::EMPTY
};
