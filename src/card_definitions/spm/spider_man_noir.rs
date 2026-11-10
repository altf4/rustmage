// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureAttacksAlone,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Entering,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::SurveilScaled(AmountValue::EventObjectTotalCounters),
        ],
    }],
    ..CardRules::EMPTY
};
