// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersManaSpent {
                color: 0,
                minimum: 2,
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: 3,
            })),
            program: &[Instruction::ReturnToBattlefield],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersManaSpent {
                color: 2,
                minimum: 2,
            },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            }),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::MinusOne,
                amount: 3,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEvokedEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::SacrificeSourceByController],
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
    extras: &CardExtras {
        evoke_costs: &[
            ManaCost {
                colored: [0, 0, 2, 0, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [1, 0, 1, 0, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 0,
            },
        ],
        cost_alternatives: &[
            ManaCost {
                colored: [1, 0, 1, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 0, 2, 0, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [1, 0, 1, 0, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
