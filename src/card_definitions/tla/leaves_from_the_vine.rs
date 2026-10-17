// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
                Instruction::CreateTokens {
                    token: Card(5246),
                    amount: 1,
                    tapped: false,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 2,
            }),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::If {
                    guard: StepGuard::Expression {
                        value: StatValue::CountUnion(&[
                            CountFilter {
                                zone: CountZone::Graveyard,
                                relation: 0,
                                kind: 1,
                                exclude: 0,
                                subtype: None,
                                basic: false,
                                other: false,
                                cards_only: true,
                            },
                            CountFilter {
                                zone: CountZone::Graveyard,
                                relation: 0,
                                kind: 0,
                                exclude: 0,
                                subtype: Some("Lesson"),
                                basic: false,
                                other: false,
                                cards_only: true,
                            },
                        ]),
                        minimum: Some(1),
                        maximum: None,
                    },
                    skip: 1,
                },
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
    ],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
