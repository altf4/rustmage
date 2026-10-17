// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::AddCombatRed(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Sacrificed {
                filter: SacrificeFilter {
                    artifact_enchantment_or_token: false,
                    keywords: Keywords(0),
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: true,
                    tokens: None,
                    subtype: None,
                },
                own_turn: false,
            },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::GrowCreatures {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords(0),
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                power: 1,
                toughness: 0,
            }],
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
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 1, 1, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 0, 2, 0, 0, 0],
                generic: 2,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
