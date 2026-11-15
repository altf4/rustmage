// Pinned Oracle and complete XMage class: only actual nontoken artifact entrants count.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersYou(EntryPredicate::NontokenArtifact),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Quest,
                amount: 1,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::SourceCounters(CounterKind::Quest)),
                minimum: 6,
                skip: 2,
            },
            Instruction::CreateTokens {
                token: Card(6007),
                amount: 2,
                tapped: false,
            },
            Instruction::Skip(1),
            Instruction::CreateTokens {
                token: Card(6007),
                amount: 1,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
