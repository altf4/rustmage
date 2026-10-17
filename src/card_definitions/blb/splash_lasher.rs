// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfOffspring,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::CreateTokens {
                token: Card(5242),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            }),
            program: &[
                Instruction::Tap,
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::Stun,
                    amount: 1,
                },
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
    keywords: Keywords(0),
    extras: &CardExtras {
        offspring_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        }),
        offspring_copy: Some(Card(5242)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
