// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::CreateTokens {
            token: Card(5295),
            amount: 2,
            tapped: false,
        }],
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
    keywords: Keywords(0),
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
                        value: StatValue::Count(CountFilter {
                            zone: CountZone::Battlefield,
                            relation: 0,
                            kind: 0,
                            exclude: 0,
                            subtype: Some("Sliver"),
                            basic: false,
                            other: false,
                            cards_only: false,
                        }),
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
            fragments: &[StaticAbilityFragment::KeywordGrant(
                0,
                KeywordGrant {
                    condition: None,
                    requires_life_gain: false,
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: Some("Sliver"),
                        keyword: Keywords(0),
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    source_only: false,
                    own_turn: false,
                    keywords: Keywords(1028),
                },
            )],
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
                            value: StatValue::Count(CountFilter {
                                zone: CountZone::Battlefield,
                                relation: 0,
                                kind: 0,
                                exclude: 0,
                                subtype: Some("Sliver"),
                                basic: false,
                                other: false,
                                cards_only: false,
                            }),
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
                fragments: &[StaticAbilityFragment::KeywordGrant(
                    0,
                    KeywordGrant {
                        condition: None,
                        requires_life_gain: false,
                        filter: CreatureFilter {
                            relation: 0,
                            other: false,
                            subtype: Some("Sliver"),
                            keyword: Keywords(0),
                            attacking: false,
                            tokens: None,
                            artifact: false,
                        },
                        source_only: false,
                        own_turn: false,
                        keywords: Keywords(1028),
                    },
                )],
            },
        ],
        keyword_grants: &[KeywordGrant {
            condition: None,
            requires_life_gain: false,
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: Some("Sliver"),
                keyword: Keywords(0),
                attacking: false,
                tokens: None,
                artifact: false,
            },
            source_only: false,
            own_turn: false,
            keywords: Keywords(1028),
        }],
        cost_reductions: &[SpellReduction {
            source_only: true,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Stat {
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Sliver"),
                    basic: false,
                    other: false,
                    cards_only: false,
                }),
                factor: 1,
            },
            turn: CostTurn::Any,
            guard: StepGuard::Always,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
