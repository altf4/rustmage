// Directly authored from complete Oracle and pinned XMage implementations.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherCreatureCountersAdded {
            kind: CounterKind::PlusOne,
            excluded_subtype: "Hydra",
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    extras: &CardExtras {
        x_symbols: 1,
        x_entry_counters: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
