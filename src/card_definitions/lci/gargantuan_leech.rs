// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(64),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CostReduction(
                0,
                SpellReduction {
                    source_only: true,
                    filter: CastFilter::Any,
                    amount: SpellReductionAmount::Stat {
                        value: StatValue::Sum(&[
                            StatValue::Count(CountFilter {
                                zone: CountZone::Battlefield,
                                relation: 0,
                                kind: 0,
                                exclude: 0,
                                subtype: Some("Cave"),
                                basic: false,
                                other: false,
                                cards_only: false,
                            }),
                            StatValue::Count(CountFilter {
                                zone: CountZone::Graveyard,
                                relation: 0,
                                kind: 0,
                                exclude: 0,
                                subtype: Some("Cave"),
                                basic: false,
                                other: false,
                                cards_only: true,
                            }),
                        ]),
                        factor: 1,
                    },
                    turn: CostTurn::Any,
                    guard: StepGuard::Always,
                },
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(64))],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Rule,
                fragments: &[StaticAbilityFragment::CostReduction(
                    0,
                    SpellReduction {
                        source_only: true,
                        filter: CastFilter::Any,
                        amount: SpellReductionAmount::Stat {
                            value: StatValue::Sum(&[
                                StatValue::Count(CountFilter {
                                    zone: CountZone::Battlefield,
                                    relation: 0,
                                    kind: 0,
                                    exclude: 0,
                                    subtype: Some("Cave"),
                                    basic: false,
                                    other: false,
                                    cards_only: false,
                                }),
                                StatValue::Count(CountFilter {
                                    zone: CountZone::Graveyard,
                                    relation: 0,
                                    kind: 0,
                                    exclude: 0,
                                    subtype: Some("Cave"),
                                    basic: false,
                                    other: false,
                                    cards_only: true,
                                }),
                            ]),
                            factor: 1,
                        },
                        turn: CostTurn::Any,
                        guard: StepGuard::Always,
                    },
                )],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(64))],
            },
        ],
        cost_reductions: &[SpellReduction {
            source_only: true,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Stat {
                value: StatValue::Sum(&[
                    StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 0,
                        exclude: 0,
                        subtype: Some("Cave"),
                        basic: false,
                        other: false,
                        cards_only: false,
                    }),
                    StatValue::Count(CountFilter {
                        zone: CountZone::Graveyard,
                        relation: 0,
                        kind: 0,
                        exclude: 0,
                        subtype: Some("Cave"),
                        basic: false,
                        other: false,
                        cards_only: true,
                    }),
                ]),
                factor: 1,
            },
            turn: CostTurn::Any,
            guard: StepGuard::Always,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
