// Pinned Oracle and complete XMage class: the threshold follows counter placement.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::FIRST_STRIKE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBecomesTapped,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Promotion,
                amount: 1,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::SourceCounters(CounterKind::Promotion)),
                minimum: 3,
                skip: 2,
            },
            Instruction::RemoveAllSourceCounters(CounterKind::Promotion),
            Instruction::DamagePlayers(PlayerSet::Opponent, 2),
        ],
    }],
    ..CardRules::EMPTY
};
