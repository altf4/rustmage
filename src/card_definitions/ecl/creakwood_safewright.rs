// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    enters: Counters {
        minus_one: 3,
        plus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::SourceCounters {
                kind: CounterKind::MinusOne,
                minimum: Some(1),
                maximum: None,
            },
            event: &TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Expression {
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Graveyard,
                        relation: 0,
                        kind: 0,
                        exclude: 0,
                        subtype: Some("Elf"),
                        basic: false,
                        other: false,
                        cards_only: true,
                    }),
                    minimum: Some(1),
                    maximum: None,
                },
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::RemoveSourceCounterIfAny {
            kind: CounterKind::MinusOne,
            skip: 0,
        }],
    }],
    ..CardRules::EMPTY
};
