// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[
                Instruction::Tap,
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::Stun,
                    amount: 1,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::ManaSpent {
                minimum: 0,
                increment: false,
                event: &TriggerCondition::SpellCast {
                    relation: CastRelation::You,
                    filter: CastFilter::InstantSorcery,
                    ordinal: None,
                },
            },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::IfManaSpent {
                    minimum: None,
                    maximum: Some(4),
                    skip: 1,
                },
                Instruction::Grow {
                    subject: Subject::Source,
                    power: 1,
                    toughness: 1,
                },
                Instruction::IfManaSpent {
                    minimum: Some(5),
                    maximum: None,
                    skip: 1,
                },
                Instruction::Grow {
                    subject: Subject::Source,
                    power: 2,
                    toughness: 2,
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
    ..CardRules::EMPTY
};
