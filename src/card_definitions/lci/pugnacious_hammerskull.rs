// The Dinosaur check belongs to the declaration event.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            event: &TriggerCondition::SelfAttacks,
            guard: StepGuard::Expression {
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Dinosaur"),
                    basic: false,
                    other: true,
                    cards_only: false,
                }),
                minimum: None,
                maximum: Some(0),
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::Stun,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
