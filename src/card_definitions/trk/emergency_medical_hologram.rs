// Pinned Oracle and XMage: the life gain follows the targeted counter ability.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Creature),
        program: &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
