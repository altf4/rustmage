// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 2,
        }),
        program: &[Instruction::Tap],
    }],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::OpponentsTurn,
    keywords: Keywords(32),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32))],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CostReduction(
                0,
                SpellReduction {
                    source_only: true,
                    filter: CastFilter::Any,
                    amount: SpellReductionAmount::Stat {
                        value: StatValue::Count(CountFilter {
                            zone: CountZone::Graveyard,
                            relation: 0,
                            kind: 12,
                            exclude: 0,
                            subtype: None,
                            basic: false,
                            other: false,
                            cards_only: true,
                        }),
                        factor: 1,
                    },
                    turn: CostTurn::Any,
                    guard: StepGuard::Always,
                },
            )],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32))],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Rule,
                fragments: &[StaticAbilityFragment::CostReduction(
                    0,
                    SpellReduction {
                        source_only: true,
                        filter: CastFilter::Any,
                        amount: SpellReductionAmount::Stat {
                            value: StatValue::Count(CountFilter {
                                zone: CountZone::Graveyard,
                                relation: 0,
                                kind: 12,
                                exclude: 0,
                                subtype: None,
                                basic: false,
                                other: false,
                                cards_only: true,
                            }),
                            factor: 1,
                        },
                        turn: CostTurn::Any,
                        guard: StepGuard::Always,
                    },
                )],
            },
        ],
        cost_reductions: &[SpellReduction {
            source_only: true,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Stat {
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 12,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
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
