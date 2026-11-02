// Count the full current graveyard after milling, with artifact/creature union deduplication.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
            Instruction::PutCountersScaled {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 17,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                })),
            },
        ],
    }],
    ..CardRules::EMPTY
};
