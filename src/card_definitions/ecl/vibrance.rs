// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersManaSpent {
                color: 3,
                minimum: 2,
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
            program: &[Instruction::Damage(3)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersManaSpent {
                color: 4,
                minimum: 2,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Search(SearchSpec {
                    filter: SearchFilter::Land,
                    maximum: 1,
                    destination: SearchDestination::Hand,
                    tapped: false,
                    reveal: true,
                }),
                Instruction::GainLife(PlayerSet::Controller, 2),
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
                colored: [0, 0, 0, 0, 2, 0],
                generic: 0,
            },
            ManaCost {
                colored: [0, 0, 0, 1, 1, 0],
                generic: 0,
            },
            ManaCost {
                colored: [0, 0, 0, 2, 0, 0],
                generic: 0,
            },
        ],
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 0, 1, 1, 0],
                generic: 3,
            },
            ManaCost {
                colored: [0, 0, 0, 2, 0, 0],
                generic: 3,
            },
            ManaCost {
                colored: [0, 0, 0, 0, 2, 0],
                generic: 0,
            },
            ManaCost {
                colored: [0, 0, 0, 1, 1, 0],
                generic: 0,
            },
            ManaCost {
                colored: [0, 0, 0, 2, 0, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
