// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ScalingModifier(
                0,
                ScalingModifier {
                    subject: Subject::Source,
                    value: StatValue::AtLeast {
                        value: &StatValue::DistinctCreatureCounterKinds,
                        minimum: 3,
                    },
                    stats: Stats {
                        power: 2,
                        toughness: 4,
                    },
                },
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::SelfGraveyardCastIf(
                StatValue::Fixed(1),
            )],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Modify,
                fragments: &[StaticAbilityFragment::ScalingModifier(
                    0,
                    ScalingModifier {
                        subject: Subject::Source,
                        value: StatValue::AtLeast {
                            value: &StatValue::DistinctCreatureCounterKinds,
                            minimum: 3,
                        },
                        stats: Stats {
                            power: 2,
                            toughness: 4,
                        },
                    },
                )],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Rule,
                fragments: &[StaticAbilityFragment::SelfGraveyardCastIf(
                    StatValue::Fixed(1),
                )],
            },
        ],
        scaling_modifiers: &[ScalingModifier {
            subject: Subject::Source,
            value: StatValue::AtLeast {
                value: &StatValue::DistinctCreatureCounterKinds,
                minimum: 3,
            },
            stats: Stats {
                power: 2,
                toughness: 4,
            },
        }],
        graveyard_cast_entry_counters: &[(CounterKind::Finality, 1)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
