// Oracle and official Star Trek release notes; pinned XMage class is absent.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LandEntersYou,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Quest,
                amount: 1,
            },
            Instruction::ExileUntilNonlandPlayThisTurn,
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::SourceCounters(CounterKind::Quest)),
                minimum: 5,
                skip: 1,
            },
            Instruction::AddMana(ManaProduction::Fixed([0, 0, 0, 2, 0, 0])),
        ],
    }],
    ..CardRules::EMPTY
};
