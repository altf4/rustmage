// Direct complete typed Oracle implementation using the shared counter core.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Put a +1/+1 counter on Mister Hyde.",
                targets: TargetPlan::NONE,
                program: &[Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                }],
            },
            SpellMode {
                text: "Remove a counter from a creature you control. If you do, draw a card.",
                targets: TargetPlan::NONE,
                program: &[Instruction::RemoveCreatureCounterThenDraw],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
