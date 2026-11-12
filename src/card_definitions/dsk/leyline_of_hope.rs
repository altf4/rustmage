// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::LifeGainAdditional(1)],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ControlledCreatureScaling {
                subtype: None,
                value: StatValue::ControllerLifeAboveStarting { minimum: 7 },
                stats: Stats {
                    power: 2,
                    toughness: 2,
                },
            }],
        },
    ],
    extras: &CardExtras {
        opening_hand_battlefield: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
