// Direct Oracle behavior; removal and simultaneous counter placement are distinct.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 0),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayRemoveSourceCounter {
                kind: CounterKind::PlusOne,
                skip: 2,
            },
            Instruction::RemoveSourceCounterIfAny {
                kind: CounterKind::PlusOne,
                skip: 1,
            },
            Instruction::PutOtherControlledCreaturesExceptSource {
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    }],
    extras: &CardExtras {
        x_symbols: 1,
        x_entry_counters: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
