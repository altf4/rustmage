// Creation completes, then counters are put on the exact resulting tokens, then the source.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CreateTokensWithCounters {
                token: Card(5395),
                amount: 1,
                kind: CounterKind::PlusOne,
                counters: AmountValue::SourceCounterQuantity(CounterKind::Invasion),
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Invasion,
                amount: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
