// Pinned Oracle and XMage: each completed controller draw adds one counter.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardDrawn {
            relation: CastRelation::You,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
