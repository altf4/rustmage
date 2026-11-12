// Direct typed behavior from pinned Oracle and XMage sources.
const SQUIRRELS: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Squirrel"),
    basic: false,
    other: true,
    cards_only: false,
};
const FOOD: CountFilter = CountFilter {
    subtype: Some("Food"),
    ..SQUIRRELS
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::TRAMPLE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCountersScaled {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: AmountValue::Stat(StatValue::CountUnion(&[SQUIRRELS, FOOD])),
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::OtherFilteredEntersYou(EntryPredicate::SubtypeAny(&[
                "Squirrel", "Food",
            ])),
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
