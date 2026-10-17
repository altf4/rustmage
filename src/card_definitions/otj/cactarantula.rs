// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTargeted {
            relation: CastRelation::Opponent,
            spells_only: false,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::May { skip: 1 },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
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
    keywords: Keywords(2),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CostReduction(
                0,
                SpellReduction {
                    source_only: true,
                    filter: CastFilter::Any,
                    amount: SpellReductionAmount::Fixed(1),
                    turn: CostTurn::Any,
                    guard: StepGuard::Expression {
                        value: StatValue::Count(CountFilter {
                            zone: CountZone::Battlefield,
                            relation: 0,
                            kind: 0,
                            exclude: 0,
                            subtype: Some("Desert"),
                            basic: false,
                            other: false,
                            cards_only: false,
                        }),
                        minimum: Some(1),
                        maximum: None,
                    },
                },
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(2))],
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
                        amount: SpellReductionAmount::Fixed(1),
                        turn: CostTurn::Any,
                        guard: StepGuard::Expression {
                            value: StatValue::Count(CountFilter {
                                zone: CountZone::Battlefield,
                                relation: 0,
                                kind: 0,
                                exclude: 0,
                                subtype: Some("Desert"),
                                basic: false,
                                other: false,
                                cards_only: false,
                            }),
                            minimum: Some(1),
                            maximum: None,
                        },
                    },
                )],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(2))],
            },
        ],
        cost_reductions: &[SpellReduction {
            source_only: true,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Fixed(1),
            turn: CostTurn::Any,
            guard: StepGuard::Expression {
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Desert"),
                    basic: false,
                    other: false,
                    cards_only: false,
                }),
                minimum: Some(1),
                maximum: None,
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
