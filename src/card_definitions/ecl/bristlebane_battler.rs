// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::TRAMPLE,
    enters: Counters {
        plus_one: 0,
        minus_one: 5,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 2,
                },
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::SourceCounters {
                    kind: CounterKind::MinusOne,
                    minimum: Some(1),
                    maximum: None,
                },
                event: &TriggerCondition::OtherCreatureEntersYou,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::RemoveSourceCounterIfAny {
                kind: CounterKind::MinusOne,
                skip: 0,
            }],
        },
    ],
    ..CardRules::EMPTY
};
