// Directly authored from pinned Oracle and XMage; pending focused validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ScryOrSurveil,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 2,
                },
                skip: 2,
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],

    ..CardRules::EMPTY
};
