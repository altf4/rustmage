// Direct typed behavior; complete Oracle body and pinned XMage sources reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    enters: Counters {
        minus_one: 3,
        plus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::SourceCounters {
                kind: CounterKind::MinusOne,
                minimum: Some(1),
                maximum: None,
            },
            event: &TriggerCondition::SelfBecomesTapped,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::RemoveSourceCounterIfAny {
            kind: CounterKind::MinusOne,
            skip: 0,
        }],
    }],
    ..CardRules::EMPTY
};
