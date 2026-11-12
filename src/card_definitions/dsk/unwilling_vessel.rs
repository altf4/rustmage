pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::VIGILANCE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Eerie,
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Possession,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateSizedToken {
                token: Card::from_catalogue_id(5675),
                size: AmountValue::SourceTotalCounters,
            }],
        },
    ],
    ..CardRules::EMPTY
};
