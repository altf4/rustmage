// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    enters: Counters {
        plus_one: 1,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfLeaves,
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::PutCountersScaled {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: AmountValue::SourceCounterQuantity(CounterKind::PlusOne),
        }],
    }],
    ..CardRules::EMPTY
};
