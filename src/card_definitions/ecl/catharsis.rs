// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersManaSpent {
                color: 0,
                minimum: 2,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5306),
                amount: 2,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersManaSpent {
                color: 3,
                minimum: 2,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::GrowCreatures {
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: None,
                        keyword: Keywords::NONE,
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    power: 1,
                    toughness: 1,
                },
                Instruction::GrantCreatures {
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: None,
                        keyword: Keywords::NONE,
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    keywords: Keywords::HASTE,
                },
            ],
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
                colored: [0, 0, 0, 2, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [1, 0, 0, 1, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 0,
            },
        ],
        cost_alternatives: &[
            ManaCost {
                colored: [1, 0, 0, 1, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 0, 0, 2, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [1, 0, 0, 1, 0, 0],
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
