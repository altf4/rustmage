// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
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
                power: 2,
                toughness: 2,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Creatures {
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: Some("Ally"),
                        keyword: Keywords(0),
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    qualification: DamageQualification::Any,
                },
                combat_only: true,
                opponent_only: false,
                one_or_more: true,
            },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
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
    ..CardRules::EMPTY
};
