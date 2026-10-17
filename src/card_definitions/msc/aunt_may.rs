// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherCreatureEntersYou,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::GainLife(PlayerSet::Controller, 1),
            Instruction::IfSubject {
                subject: Subject::Entering,
                filter: CreatureFilter {
                    relation: -1,
                    other: false,
                    subtype: Some("Spider"),
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                skip: 1,
            },
            Instruction::PutCounters {
                subject: Subject::Entering,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
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
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
