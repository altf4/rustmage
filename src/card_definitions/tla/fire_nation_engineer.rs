// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Attacked,
        },
        targets: one(TargetRule::Filtered {
            base: &TargetRule::CreatureOrVehicle { relation: 0 },
            predicate: &TargetPredicate::OtherThanSource,
        }),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
