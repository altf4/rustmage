// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    keywords: Keywords::SKULK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            event: &TriggerCondition::AtStep {
                step: TriggerStep::Upkeep,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            guard: StepGuard::Expression {
                value: StatValue::SourceCounters(CounterKind::PlusOne),
                minimum: Some(1),
                maximum: None,
            },
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
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    extras: &CardExtras {
        x_symbols: 1,
        x_entry_counters: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
