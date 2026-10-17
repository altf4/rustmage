// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Mill(
            PlayerSet::Controller,
            MillAmount::Fixed(3),
        )],
    }],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(1),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(1))],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ScalingModifier(
                0,
                ScalingModifier {
                    subject: Subject::Source,
                    value: StatValue::AtLeast {
                        value: &StatValue::Count(CountFilter {
                            zone: CountZone::Graveyard,
                            relation: 0,
                            kind: 0,
                            exclude: 0,
                            subtype: None,
                            basic: false,
                            other: false,
                            cards_only: true,
                        }),
                        minimum: 7,
                    },
                    stats: Stats {
                        power: 2,
                        toughness: 1,
                    },
                },
            )],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(1))],
            },
            StaticAbilityGroup {
                line: 2,
                first_layer: AbilityLayer::Modify,
                fragments: &[StaticAbilityFragment::ScalingModifier(
                    0,
                    ScalingModifier {
                        subject: Subject::Source,
                        value: StatValue::AtLeast {
                            value: &StatValue::Count(CountFilter {
                                zone: CountZone::Graveyard,
                                relation: 0,
                                kind: 0,
                                exclude: 0,
                                subtype: None,
                                basic: false,
                                other: false,
                                cards_only: true,
                            }),
                            minimum: 7,
                        },
                        stats: Stats {
                            power: 2,
                            toughness: 1,
                        },
                    },
                )],
            },
        ],
        scaling_modifiers: &[ScalingModifier {
            subject: Subject::Source,
            value: StatValue::AtLeast {
                value: &StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                }),
                minimum: 7,
            },
            stats: Stats {
                power: 2,
                toughness: 1,
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
