// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::CounterBearingCreatures),
                minimum: 1,
                skip: 1,
            },
            Instruction::DrawRemembered(AmountValue::Fixed(1)),
            Instruction::IfNoRememberedDraws { skip: 1 },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
